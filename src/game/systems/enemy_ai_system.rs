use avian2d::prelude::Gravity as WorldGravity;
use bevy::prelude::*;
// EventWriter avoided; using JumpParticlesQueue resource instead
use rand::seq::SliceRandom;

use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::{
    AutoMovement, AutoMovementAggroStrategy, AutoMovementDefaultStrategy, AutoMovementState,
    Blocking, Collider, ColliderShape, ControlledMovement, Gravity, Health, RigidBody,
    StateMachine, Team,
};
// JumpParticlesEvent is used via the JumpParticlesQueue; no direct import needed here.
use crate::game::runtime_components::PatrolState;
const PATROL_MIN_INTERVAL_SEC: f32 = 0.8;
const PATROL_MAX_INTERVAL_SEC: f32 = 2.4;
use crate::game::tags::EnemyTag;

const EPS: f32 = 0.0001;
const JUMP_CLEARANCE: f32 = 6.0;
const JUMP_MIN_HEIGHT: f32 = 8.0;
const JUMP_LOOKAHEAD_MIN: f32 = 12.0;
const JUMP_LOOKAHEAD_MAX: f32 = 140.0;
const JUMP_OVERLAP_EPS: f32 = 2.0;
const FOLLOW_STOP_EPS: f32 = 2.0;

pub fn enemy_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    world_gravity: Res<WorldGravity>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            Option<&Collider>,
            &mut AutoMovement,
            &mut RigidBody,
            Option<&mut PatrolState>,
            Option<&mut Gravity>,
            Option<&Health>,
            Option<&AutoRangeAttack>,
            Option<&Team>,
            Option<&StateMachine>,
        ),
        With<EnemyTag>,
    >,
    targets: Query<
        (
            Entity,
            &Transform,
            Option<&Collider>,
            Option<&Team>,
            Option<&StateMachine>,
        ),
        With<ControlledMovement>,
    >,
    blockers: Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
    mut jump_queue: ResMut<crate::game::gfx::jump::JumpParticlesQueue>,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs();
    let world_gravity = Vec2::new(world_gravity.0.x, world_gravity.0.y);

    let mut share_events: Vec<(String, Entity, f32, Vec2)> = Vec::new();

    for (
        entity,
        transform,
        collider,
        mut auto,
        mut rigid_body,
        patrol_state,
        mut gravity,
        health,
        range_attack,
        team,
        sm,
    ) in &mut enemies
    {
        if sm.is_some_and(|s| s.is_non_interactive()) {
            auto.direction = Vec2::ZERO;
            rigid_body.velocity.x = 0.0;
            continue;
        }

        if !auto.has_origin {
            auto.origin = transform.translation.truncate();
            auto.has_origin = true;
        }
        if auto.deaggro_range <= auto.aggro_range {
            auto.deaggro_range = auto.aggro_range + 0.01;
        }

        auto.jump_cooldown_remaining = (auto.jump_cooldown_remaining - dt).max(0.0);
        auto.patrol_pause_remaining = (auto.patrol_pause_remaining - dt).max(0.0);
        auto.vision_tick_remaining -= dt;

        if !auto.enabled {
            auto.direction = Vec2::ZERO;
            rigid_body.velocity.x = 0.0;
            continue;
        }

        let enemy_center = world_center(transform, collider);
        tracing::debug!(entity = ?entity, state = ?auto.state, direction = ?auto.direction, strategy = ?auto.default_strategy, speed = ?auto.speed, patrol_state_present = ?patrol_state.is_some(), "enemy_ai: pre-update");
        let attacker_team_name = team.map(|t| t.name.as_str()).unwrap_or("Neutral");

        if auto.aggro && auto.vision_tick_remaining <= 0.0 {
            auto.vision_tick_remaining = auto.vision_check_interval.max(0.01);
            let mut valid_targets: Vec<(Entity, Vec2)> = Vec::new();
            for (target_entity, target_tf, target_col, target_team, target_sm) in &targets {
                if target_sm.is_some_and(|s| s.is_non_interactive()) {
                    continue;
                }
                let target_team_name = target_team.map(|t| t.name.as_str()).unwrap_or("Neutral");
                if attacker_team_name == target_team_name {
                    continue;
                }
                let target_center = world_center(target_tf, target_col);
                let to_target = target_center - enemy_center;
                let dist_sq = to_target.length_squared();
                if dist_sq > auto.aggro_range * auto.aggro_range {
                    continue;
                }
                if !within_vision_cone(auto.direction, to_target, auto.vision_angle) {
                    continue;
                }
                if auto.line_of_sight && has_los_block(enemy_center, target_center, &blockers) {
                    continue;
                }
                valid_targets.push((target_entity, target_center));
            }

            if let Some((picked_entity, picked_pos)) = valid_targets.choose(&mut rand::thread_rng())
            {
                auto.target_entity = Some(*picked_entity);
                auto.last_known_target_pos = Some(*picked_pos);
                auto.last_target_seen_secs = now;
                auto.state = AutoMovementState::Aggro;
                if let Some(team_name) = auto.share_aggro_with_team.clone() {
                    share_events.push((
                        team_name,
                        *picked_entity,
                        auto.aggro_sharing_radius,
                        enemy_center,
                    ));
                }
            }
        }

        let mut has_visible_target = false;
        let mut target_pos = auto.last_known_target_pos;
        if let Some(target_entity) = auto.target_entity {
            if let Ok((_, target_tf, target_col, _, target_sm)) = targets.get(target_entity) {
                if !target_sm.is_some_and(|s| s.is_non_interactive()) {
                    let candidate_pos = world_center(target_tf, target_col);
                    let dist_sq = enemy_center.distance_squared(candidate_pos);
                    if dist_sq > auto.deaggro_range * auto.deaggro_range {
                        // Hard de-aggro once the target leaves deaggro range.
                        auto.target_entity = None;
                        auto.last_known_target_pos = None;
                        auto.state = AutoMovementState::ReturnToOrigin;
                    } else {
                        if !auto.line_of_sight
                            || !has_los_block(enemy_center, candidate_pos, &blockers)
                        {
                            has_visible_target = true;
                            target_pos = Some(candidate_pos);
                            auto.last_known_target_pos = Some(candidate_pos);
                            auto.last_target_seen_secs = now;
                        }
                    }
                }
            }
        }

        if auto.target_entity.is_some() {
            let forgot_target = now - auto.last_target_seen_secs > auto.target_timeout;
            if forgot_target {
                auto.target_entity = None;
                auto.state = AutoMovementState::ReturnToOrigin;
            }
        }

        if auto.target_entity.is_none() && matches!(auto.state, AutoMovementState::Aggro) {
            auto.state = AutoMovementState::ReturnToOrigin;
        }

        if matches!(auto.state, AutoMovementState::Idle) {
            auto.state = match auto.default_strategy {
                AutoMovementDefaultStrategy::StandStill => AutoMovementState::Idle,
                _ => AutoMovementState::Patrol,
            };
        }

        // Attach runtime PatrolState for RandomPatrol entities if missing so
        // they can sample independent RNG streams and avoid synchronized
        // behaviour. Inserting is done here (instead of a separate system)
        // so PatrolState is available in subsequent frames without adding
        // another scheduled system.
        if auto.default_strategy == AutoMovementDefaultStrategy::RandomPatrol
            && patrol_state.is_none()
        {
            commands.entity(entity).insert(PatrolState::from_entity(entity));
        }

        match auto.state {
            AutoMovementState::Idle => {
                auto.direction = Vec2::ZERO;
                rigid_body.velocity.x = 0.0;
            }
            AutoMovementState::Patrol => {
                tracing::debug!(entity = ?entity, "enemy_ai: entering Patrol (strategy={:?}, patrol_state_present={})", auto.default_strategy, patrol_state.is_some());
                apply_patrol(
                    enemy_center,
                    &mut auto,
                    &mut rigid_body,
                    gravity.as_deref_mut(),
                    patrol_state,
                    world_gravity,
                    dt,
                    collider,
                    &blockers,
                    &mut *jump_queue,
                );
                tracing::debug!(entity = ?entity, direction = ?auto.direction, vx = ?rigid_body.velocity.x, "enemy_ai: after Patrol");
            }
            AutoMovementState::Aggro => {
                    if let Some(tpos) = target_pos {
                            apply_aggro(
                                enemy_center,
                                tpos,
                                has_visible_target,
                                &mut auto,
                                &mut rigid_body,
                                gravity.as_deref_mut(),
                                health,
                                range_attack,
                                collider,
                                &blockers,
                                world_gravity,
                                dt,
                                &mut *jump_queue,
                            );
                } else {
                    auto.state = AutoMovementState::ReturnToOrigin;
                }
            }
            AutoMovementState::ReturnToOrigin => {
                let delta_x = auto.origin.x - enemy_center.x;
                if delta_x.abs() <= 2.0 {
                    auto.direction = Vec2::ZERO;
                    rigid_body.velocity.x = 0.0;
                    auto.state = match auto.default_strategy {
                        AutoMovementDefaultStrategy::StandStill => AutoMovementState::Idle,
                        _ => AutoMovementState::Patrol,
                    };
                } else {
                    auto.direction = Vec2::new(delta_x.signum(), 0.0);
                    accelerate_x(
                        &mut rigid_body,
                        auto.direction.x * auto.max_speed,
                        auto.acceleration,
                        dt,
                    );
                    attempt_navigation_jump(
                        enemy_center,
                        Some(auto.origin),
                        &mut auto,
                        &mut rigid_body,
                        gravity.as_deref_mut(),
                        collider,
                        &blockers,
                        world_gravity,
                        &mut *jump_queue,
                    );
                }
            }
        }
    }

    if !share_events.is_empty() {
        for (_entity, transform, _collider, mut auto, _rb, _patrol_state, _g, _h, _ra, team, _sm) in &mut enemies {
            let Some(team_name) = team.map(|t| t.name.as_str()) else {
                continue;
            };
            let my_pos = transform.translation.truncate();
            for (share_team, target_entity, share_radius, origin) in &share_events {
                if team_name == share_team.as_str()
                    && my_pos.distance_squared(*origin) <= *share_radius * *share_radius
                {
                    auto.target_entity = Some(*target_entity);
                    auto.state = AutoMovementState::Aggro;
                    auto.last_target_seen_secs = now;
                }
            }
        }
    }
}

fn apply_patrol(
    enemy_center: Vec2,
    auto: &mut AutoMovement,
    rigid_body: &mut RigidBody,
    gravity: Option<&mut Gravity>,
    patrol_state: Option<bevy::prelude::Mut<'_, PatrolState>>,
    world_gravity: Vec2,
    dt: f32,
    collider: Option<&Collider>,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
    jump_queue: &mut crate::game::gfx::jump::JumpParticlesQueue,
) {
    // Consume any patrol pause set by other logic (e.g. waypoint arrival or
    // range flip).
    if auto.patrol_pause_remaining > 0.0 {
        auto.patrol_pause_remaining = (auto.patrol_pause_remaining - dt).max(0.0);
        auto.direction = Vec2::ZERO;
        rigid_body.velocity.x = 0.0;
        return;
    }

    match auto.default_strategy {
        AutoMovementDefaultStrategy::StandStill => {
            auto.direction = Vec2::ZERO;
            rigid_body.velocity.x = 0.0;
        }
        AutoMovementDefaultStrategy::RandomPatrol => {
            // If a PatrolState is present, use its RNG to occasionally change
            // the chosen direction so movement appears more random rather
            // than strictly one-direction.
            if let Some(mut ps) = patrol_state {
                ps.timer -= dt;
                if ps.timer <= 0.0 {
                    let rv = ps.next_rand();
                    // 40% left, 40% right, 20% pause
                    ps.direction = if rv < 0.4 { -1.0 } else if rv > 0.6 { 1.0 } else { 0.0 };
                    let interval_rand = ps.next_rand();
                    ps.timer = PATROL_MIN_INTERVAL_SEC +
                        (PATROL_MAX_INTERVAL_SEC - PATROL_MIN_INTERVAL_SEC) * interval_rand;
                }

                // RandomPatrol should stay on platforms by design.
                // Other strategies can still opt into falling via config.
                let prevent_fall = auto.default_strategy == AutoMovementDefaultStrategy::RandomPatrol
                    || !auto.can_fall_when_following;
                let mut dir = ps.direction;
                if dir.abs() > 0.0 && prevent_fall {
                    let has_ground = ground_ahead_exists(enemy_center, dir, collider, blockers);
                    if !has_ground {
                        // Try the opposite direction; if that is also unsafe,
                        // pause instead.
                        let alt_dir = -dir;
                        if ground_ahead_exists(enemy_center, alt_dir, collider, blockers) {
                            dir = alt_dir;
                        } else {
                            dir = 0.0;
                        }
                    }
                }

                // If outside patrol range, force movement back toward origin
                // instead of pausing indefinitely on the boundary.
                let delta = enemy_center.x - auto.origin.x;
                if auto.patrol_range > 0.0 && delta.abs() >= auto.patrol_range {
                    let inward = -delta.signum();
                    if inward.abs() > EPS {
                        dir = inward;
                        auto.patrol_direction = inward;
                    }
                }

                ps.direction = dir;
                auto.direction = Vec2::new(dir, 0.0);
            } else {
                // Fallback to the simple patrol_direction behaviour if the
                // PatrolState hasn't been attached yet.
                let mut dir = auto.patrol_direction;
                if dir.abs() > 0.0
                    && (auto.default_strategy == AutoMovementDefaultStrategy::RandomPatrol
                        || !auto.can_fall_when_following)
                {
                    if !ground_ahead_exists(enemy_center, dir, collider, blockers) {
                        let alt = -dir;
                        if ground_ahead_exists(enemy_center, alt, collider, blockers) {
                            dir = alt;
                        } else {
                            dir = 0.0;
                        }
                    }
                }

                // Keep the fallback path consistent with PatrolState behavior.
                let delta = enemy_center.x - auto.origin.x;
                if auto.patrol_range > 0.0 && delta.abs() >= auto.patrol_range {
                    let inward = -delta.signum();
                    if inward.abs() > EPS {
                        dir = inward;
                        auto.patrol_direction = inward;
                    }
                }

                auto.direction = Vec2::new(dir, 0.0);
            }

            rigid_body.velocity.x = auto.direction.x * auto.speed.max(auto.max_speed);
            attempt_navigation_jump(
                enemy_center,
                None,
                auto,
                rigid_body,
                gravity,
                collider,
                blockers,
                world_gravity,
                jump_queue,
            );
        }
        AutoMovementDefaultStrategy::WaypointsPatrol => {
            if auto.patrol_waypoints.is_empty() {
                auto.direction = Vec2::ZERO;
                rigid_body.velocity.x = 0.0;
                return;
            }
            let idx = auto.patrol_waypoint_index % auto.patrol_waypoints.len();
            let target = auto.patrol_waypoints[idx];
            let delta_x = target.x - enemy_center.x;
            if delta_x.abs() <= 2.0 {
                auto.patrol_waypoint_index = (idx + 1) % auto.patrol_waypoints.len();
                auto.patrol_pause_remaining = auto.patrol_pause_time;
                auto.direction = Vec2::ZERO;
                rigid_body.velocity.x = 0.0;
            } else {
                auto.direction = Vec2::new(delta_x.signum(), 0.0);
                rigid_body.velocity.x = auto.direction.x * auto.speed.max(auto.max_speed);
                        attempt_navigation_jump(
                            enemy_center,
                            Some(target),
                            auto,
                            rigid_body,
                            gravity,
                            collider,
                            blockers,
                            world_gravity,
                            jump_queue,
                        );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_aggro(
    enemy_center: Vec2,
    target_pos: Vec2,
    has_visible_target: bool,
    auto: &mut AutoMovement,
    rigid_body: &mut RigidBody,
    gravity: Option<&mut Gravity>,
    health: Option<&Health>,
    range_attack: Option<&AutoRangeAttack>,
    collider: Option<&Collider>,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
    world_gravity: Vec2,
    dt: f32,
    jump_queue: &mut crate::game::gfx::jump::JumpParticlesQueue,
) {
    let dx = target_pos.x - enemy_center.x;
    let desired;

    // Determine distance and health fraction used by aggro strategies.
    let distance = enemy_center.distance(target_pos);
    let hp_frac = health
        .map(|h| {
            if h.max > 0 {
                h.current as f32 / h.max as f32
            } else {
                1.0
            }
        })
        .unwrap_or(1.0);

    // Attack aggro range is attack-specific if present, otherwise fall back
    // to the movement-configured aggro_range.
    let attack_aggro_range = range_attack.map(|a| a.aggro_range).unwrap_or(auto.aggro_range);

    match auto.aggro_strategy {
        AutoMovementAggroStrategy::Kiting => {
            let low_hp_kite = auto.kiting_enabled && hp_frac <= auto.kiting_hp_threshold;
            let min_engage = auto.min_engage_distance;

            if low_hp_kite && has_visible_target {
                desired = -dx.signum();
            } else if distance < min_engage {
                desired = -dx.signum();
            } else if distance > attack_aggro_range {
                desired = dx.signum();
            } else {
                desired = 0.0;
            }
        }
        _ => {
            // Follow should keep chasing while the target is tracked.
            // `aggro_range` gates acquisition, `deaggro_range` gates loss.
            // Movement itself stops only in close distance.
            let follow_stop_distance = auto.follow_stop_distance.max(0.0);
            if distance > follow_stop_distance + FOLLOW_STOP_EPS {
                desired = dx.signum();
            } else {
                desired = 0.0;
            }
        }
    }

    auto.direction = Vec2::new(desired, 0.0);
    if auto.direction.x.abs() <= EPS {
        rigid_body.velocity.x = 0.0;
    } else {
        let target_vx = auto.direction.x * auto.max_speed.max(auto.speed);
        accelerate_x(rigid_body, target_vx, auto.acceleration, dt);
    }

    attempt_navigation_jump(
        enemy_center,
        Some(target_pos),
        auto,
        rigid_body,
        gravity,
        collider,
        blockers,
        world_gravity,
        jump_queue,
    );
}

fn accelerate_x(rb: &mut RigidBody, target_vx: f32, acceleration: f32, dt: f32) {
    let delta = target_vx - rb.velocity.x;
    let max_step = acceleration.max(0.0) * dt;
    if delta.abs() <= max_step {
        rb.velocity.x = target_vx;
    } else {
        rb.velocity.x += max_step * delta.signum();
    }
}

fn world_center(transform: &Transform, collider: Option<&Collider>) -> Vec2 {
    transform.translation.truncate() + collider.map(|c| c.offset).unwrap_or(Vec2::ZERO)
}

fn within_vision_cone(direction: Vec2, to_target: Vec2, vision_angle_deg: f32) -> bool {
    if to_target.length_squared() <= EPS {
        return true;
    }
    let forward = if direction.x.abs() > EPS {
        Vec2::new(direction.x.signum(), 0.0)
    } else {
        Vec2::X
    };
    let dir = to_target.normalize_or_zero();
    let dot = forward.dot(dir).clamp(-1.0, 1.0);
    let angle = dot.acos().to_degrees();
    angle <= (vision_angle_deg * 0.5).max(0.0)
}

fn has_los_block(
    start: Vec2,
    end: Vec2,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
) -> bool {
    for (transform, collider, blocker_sm, blocking_comp) in blockers {
        if blocker_sm.is_some_and(|sm| sm.is_non_interactive()) {
            continue;
        }
        // Only consider blockers that are configured to block line of sight.
        if !blocking_comp.blocks_line_of_sight {
            continue;
        }
        let ColliderShape::Rectangle { half_extents } = &collider.shape;
        let center = transform.translation.truncate() + collider.offset;
        let min = center - *half_extents;
        let max = center + *half_extents;
        if segment_intersects_aabb(start, end, min, max) {
            return true;
        }
    }
    false
}

fn segment_intersects_aabb(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> bool {
    let dir = end - start;
    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;

    for i in 0..2 {
        let s = if i == 0 { start.x } else { start.y };
        let d = if i == 0 { dir.x } else { dir.y };
        let mn = if i == 0 { min.x } else { min.y };
        let mx = if i == 0 { max.x } else { max.y };

        if d.abs() <= EPS {
            if s < mn || s > mx {
                return false;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (mn - s) * inv_d;
            let mut t2 = (mx - s) * inv_d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
            if t_min > t_max {
                return false;
            }
        }
    }

    true
}

fn ground_ahead_exists(
    enemy_center: Vec2,
    dir: f32,
    collider: Option<&Collider>,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
) -> bool {
    if dir.abs() <= EPS {
        return true;
    }

    // Use the entity's collider half extents or a reasonable default.
    let half_extents = collider
        .map(|c| match &c.shape {
            ColliderShape::Rectangle { half_extents } => *half_extents,
        })
        .unwrap_or(Vec2::new(8.0, 8.0));

    // Probe from just ahead of the feet downwards to detect floor at the
    // next step location.
    let probe_x = enemy_center.x + dir.signum() * (half_extents.x + 2.0);
    let foot_y = enemy_center.y - half_extents.y;
    let probe_start = Vec2::new(probe_x, foot_y + 1.0);
    let probe_end = probe_start + Vec2::new(0.0, -(half_extents.y + 8.0));

    for (t, c, blocker_sm, _blocking_comp) in blockers {
        if blocker_sm.is_some_and(|sm| sm.is_non_interactive()) {
            continue;
        }
        let ColliderShape::Rectangle { half_extents } = &c.shape;
        let center = t.translation.truncate() + c.offset;
        let min = center - *half_extents;
        let max = center + *half_extents;
        if segment_intersects_aabb(probe_start, probe_end, min, max) {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn attempt_navigation_jump(
    enemy_center: Vec2,
    navigation_target: Option<Vec2>,
    auto: &mut AutoMovement,
    rigid_body: &mut RigidBody,
    gravity: Option<&mut Gravity>,
    collider: Option<&Collider>,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
    world_gravity: Vec2,
    jump_queue: &mut crate::game::gfx::jump::JumpParticlesQueue,
) {
    if !auto.jump_enabled_for_state(auto.state)
        || auto.jump_force <= EPS
        || auto.jump_cooldown_remaining > 0.0
    {
        return;
    }

    let move_dir = auto.direction.x.signum();
    if move_dir.abs() <= EPS {
        return;
    }

    let Some(gravity) = gravity else {
        return;
    };
    if !gravity.grounded {
        return;
    }

    let jump_height = max_jump_height(auto.jump_force, gravity.scale, world_gravity);
    if jump_height <= JUMP_MIN_HEIGHT {
        return;
    }

    let horizontal_speed = auto.max_speed.max(auto.speed).max(rigid_body.velocity.x.abs());
    let lookahead = jump_lookahead_distance(
        enemy_center,
        navigation_target,
        horizontal_speed,
        auto.jump_force,
        gravity.scale,
        world_gravity,
    );

    if !has_reachable_jump_surface(enemy_center, move_dir, collider, blockers, jump_height, lookahead)
    {
        return;
    }

    rigid_body.velocity.y = rigid_body.velocity.y.max(auto.jump_force);
    gravity.grounded = false;
    auto.jump_cooldown_remaining = auto.jump_cooldown.max(0.0);
    // Emit a jump particle event so gfx can spawn dust biased by movement dir.
    let origin = enemy_center;
    let seed_base = origin.x.to_bits().wrapping_mul(31) ^ origin.y.to_bits().wrapping_mul(131) ^ 0xDEAD_BEEFu32;
    let horizontal_dir = auto.direction.x.signum();
    jump_queue.0.push(crate::game::gfx::jump::JumpParticlesEvent::Jump { origin, horizontal_dir, seed_base });
}

fn has_reachable_jump_surface(
    enemy_center: Vec2,
    move_dir: f32,
    collider: Option<&Collider>,
    blockers: &Query<(&Transform, &Collider, Option<&StateMachine>, &Blocking), With<Blocking>>,
    jump_height: f32,
    lookahead: f32,
) -> bool {
    let self_half_extents = collider_half_extents(collider);
    let foot_y = enemy_center.y - self_half_extents.y;
    let front_x = enemy_center.x + move_dir.signum() * self_half_extents.x;

    let mut best_gap = f32::INFINITY;
    let mut best_required_rise = f32::INFINITY;

    for (transform, blocker, blocker_sm, _blocking_comp) in blockers {
        if blocker_sm.is_some_and(|sm| sm.is_non_interactive()) {
            continue;
        }

        let (min, max) = collider_bounds(transform, blocker);
        let gap = if move_dir > 0.0 {
            min.x - front_x
        } else {
            front_x - max.x
        };

        if gap < -JUMP_OVERLAP_EPS || gap > lookahead {
            continue;
        }

        if max.y <= foot_y + JUMP_MIN_HEIGHT {
            continue;
        }

        let required_rise = max.y - foot_y + JUMP_CLEARANCE;
        if required_rise > jump_height {
            continue;
        }

        if gap < best_gap || ((gap - best_gap).abs() <= EPS && required_rise < best_required_rise) {
            best_gap = gap;
            best_required_rise = required_rise;
        }
    }

    best_gap.is_finite()
}

fn jump_lookahead_distance(
    enemy_center: Vec2,
    navigation_target: Option<Vec2>,
    horizontal_speed: f32,
    jump_force: f32,
    gravity_scale: f32,
    world_gravity: Vec2,
) -> f32 {
    let gravity_strength = (world_gravity.length() * gravity_scale.abs()).max(EPS);
    let flight_time = if gravity_strength > EPS {
        (2.0 * jump_force.max(0.0) / gravity_strength).max(0.0)
    } else {
        0.0
    };
    let mut lookahead = (horizontal_speed.max(24.0) * flight_time).clamp(JUMP_LOOKAHEAD_MIN, JUMP_LOOKAHEAD_MAX);

    if let Some(target) = navigation_target {
        let to_target = target - enemy_center;
        if to_target.y > JUMP_MIN_HEIGHT {
            lookahead = lookahead.max(to_target.x.abs().clamp(JUMP_LOOKAHEAD_MIN, JUMP_LOOKAHEAD_MAX));
        }
    }

    lookahead
}

fn max_jump_height(jump_force: f32, gravity_scale: f32, world_gravity: Vec2) -> f32 {
    let gravity_strength = world_gravity.length() * gravity_scale.abs();
    if gravity_strength <= EPS {
        return jump_force.max(0.0);
    }

    let clamped_jump_force = jump_force.max(0.0);
    (clamped_jump_force * clamped_jump_force) / (2.0 * gravity_strength)
}

fn collider_half_extents(collider: Option<&Collider>) -> Vec2 {
    collider
        .map(|c| match &c.shape {
            ColliderShape::Rectangle { half_extents } => *half_extents,
        })
        .unwrap_or(Vec2::new(8.0, 8.0))
}

fn collider_bounds(transform: &Transform, collider: &Collider) -> (Vec2, Vec2) {
    let ColliderShape::Rectangle { half_extents } = &collider.shape;
    let center = transform.translation.truncate() + collider.offset;
    (center - *half_extents, center + *half_extents)
}

