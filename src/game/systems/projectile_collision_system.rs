use bevy::prelude::*;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::{Blocking, Collider, ColliderShape, Damageable, Health, RigidBody, Team};
use crate::game::gfx::plasma_impact::spawn_plasma_impact_explosion;
use crate::game::gfx::plasma_shoot::{ensure_plasma_particle_image, PlasmaParticleImage};
use crate::game::runtime_components::Projectile;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::spawn_combat_sfx;

const TOI_EPSILON: f32 = 1e-4;
const NEUTRAL_TEAM: &str = "Neutral";
const PLASMA_HIT_SFX: &str = "audio/plasma-hit.ogg";
const BEAM_AFTERGLOW_SECS: f32 = 0.28;

pub fn projectile_collision_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    particle_image_res: Option<Res<PlasmaParticleImage>>,
    projectiles: Query<(Entity, &Transform, &Collider, &RigidBody, &Projectile)>,
    targets: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&Blocking>,
            Option<&RigidBody>,
            Option<&Team>,
        ),
    >,
    teams: Query<&Team>,
    mut health_query: Query<&mut Health>,
    mut damageable_query: Query<&mut Damageable>,
    mut beams: Query<(Entity, &mut PlasmaBeam)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (projectile_entity, projectile_transform, projectile_collider, projectile_body, projectile) in
        &projectiles
    {
        let Some(projectile_half_extents) = rectangle_half_extents(projectile_collider) else {
            continue;
        };

        let owner_team = teams
            .get(projectile.owner)
            .map(|team| team.name.as_str())
            .unwrap_or(NEUTRAL_TEAM);

        let projectile_center = projectile_transform.translation.truncate() + projectile_collider.offset;
        let projectile_motion = projectile_body.velocity * dt;

        let mut best_hit: Option<HitCandidate> = None;

        for (target_entity, target_transform, target_collider, blocking, target_body, target_team) in
            &targets
        {
            if target_entity == projectile.owner || target_entity == projectile_entity {
                continue;
            }

            let is_blocking = blocking.is_some();
            let is_damageable = damageable_query.contains(target_entity);
            if !is_blocking && !is_damageable {
                continue;
            }

            let target_team_name = target_team
                .map(|team| team.name.as_str())
                .unwrap_or(NEUTRAL_TEAM);
            if owner_team == target_team_name {
                continue;
            }

            let Some(target_half_extents) = rectangle_half_extents(target_collider) else {
                continue;
            };

            let target_center = target_transform.translation.truncate() + target_collider.offset;
            let target_velocity = target_body.map(|rb| rb.velocity).unwrap_or(Vec2::ZERO);
            let relative_motion = (projectile_body.velocity - target_velocity) * dt;

            let Some(toi) = swept_aabb_toi(
                projectile_center,
                projectile_half_extents,
                relative_motion,
                target_center,
                target_half_extents,
            ) else {
                continue;
            };

            let distance = projectile_motion.length() * toi;
            let class_rank = if is_blocking { 0_u8 } else { 1_u8 };
            let candidate = HitCandidate {
                entity: target_entity,
                toi,
                distance,
                class_rank,
                is_damageable,
                impact_position: projectile_center + (projectile_motion * toi),
            };

            if is_better_hit(candidate, best_hit) {
                best_hit = Some(candidate);
            }
        }

        if let Some(hit) = best_hit {
            if hit.is_damageable {
                if let Ok(mut health) = health_query.get_mut(hit.entity) {
                    health.damage(projectile.damage);
                }
                // Trigger the Damaged state by resetting the damaged timer.
                if let Ok(mut dmg) = damageable_query.get_mut(hit.entity) {
                    dmg.damaged_timer = dmg.damaged_duration_secs;
                }
            }

            if projectile
                .impact_effect
                .as_deref()
                .unwrap_or("plasma_impact")
                .eq_ignore_ascii_case("plasma_impact")
            {
                let particle_image = if let Some(resource) = particle_image_res.as_ref() {
                    resource.0.clone()
                } else {
                    ensure_plasma_particle_image(&mut plasma_particle_image, &mut images)
                };
                let impact_z = projectile_transform.translation.z;
                spawn_plasma_impact_explosion(&mut commands, &particle_image, hit.impact_position, impact_z);
            }
            spawn_combat_sfx(
                &mut commands,
                &asset_server,
                &audio_settings,
                PLASMA_HIT_SFX,
            );
            set_beams_to_afterglow(projectile_entity, &mut beams);

            commands.entity(projectile_entity).despawn();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct HitCandidate {
    entity: Entity,
    toi: f32,
    distance: f32,
    class_rank: u8,
    is_damageable: bool,
    impact_position: Vec2,
}

fn set_beams_to_afterglow(projectile_entity: Entity, beams: &mut Query<(Entity, &mut PlasmaBeam)>) {
    for (_beam_entity, mut beam) in beams {
        if beam.target_projectile == Some(projectile_entity) {
            beam.target_projectile = None;
            beam.lifetime = Some(Timer::from_seconds(BEAM_AFTERGLOW_SECS, TimerMode::Once));
        }
    }
}

fn is_better_hit(candidate: HitCandidate, current_best: Option<HitCandidate>) -> bool {
    let Some(current_best) = current_best else {
        return true;
    };

    if candidate.toi + TOI_EPSILON < current_best.toi {
        return true;
    }
    if (candidate.toi - current_best.toi).abs() > TOI_EPSILON {
        return false;
    }

    if candidate.distance + TOI_EPSILON < current_best.distance {
        return true;
    }
    if (candidate.distance - current_best.distance).abs() > TOI_EPSILON {
        return false;
    }

    if candidate.class_rank < current_best.class_rank {
        return true;
    }
    if candidate.class_rank > current_best.class_rank {
        return false;
    }

    candidate.entity.index() < current_best.entity.index()
}

fn rectangle_half_extents(collider: &Collider) -> Option<Vec2> {
    match &collider.shape {
        ColliderShape::Rectangle { half_extents } => Some(*half_extents),
        _ => None,
    }
}

fn swept_aabb_toi(
    moving_center: Vec2,
    moving_half_extents: Vec2,
    moving_delta: Vec2,
    target_center: Vec2,
    target_half_extents: Vec2,
) -> Option<f32> {
    let target_min = target_center - target_half_extents - moving_half_extents;
    let target_max = target_center + target_half_extents + moving_half_extents;

    if moving_delta.abs_diff_eq(Vec2::ZERO, f32::EPSILON) {
        if point_in_aabb(moving_center, target_min, target_max) {
            return Some(0.0);
        }
        return None;
    }

    let (tx_min, tx_max) = ray_axis_times(moving_center.x, moving_delta.x, target_min.x, target_max.x)?;
    let (ty_min, ty_max) = ray_axis_times(moving_center.y, moving_delta.y, target_min.y, target_max.y)?;

    let t_enter = tx_min.max(ty_min);
    let t_exit = tx_max.min(ty_max);

    if t_enter > t_exit || t_exit < 0.0 || t_enter > 1.0 {
        return None;
    }

    Some(t_enter.max(0.0))
}

fn ray_axis_times(origin: f32, delta: f32, slab_min: f32, slab_max: f32) -> Option<(f32, f32)> {
    if delta.abs() <= f32::EPSILON {
        if origin < slab_min || origin > slab_max {
            return None;
        }
        return Some((f32::NEG_INFINITY, f32::INFINITY));
    }

    let inv = 1.0 / delta;
    let mut t1 = (slab_min - origin) * inv;
    let mut t2 = (slab_max - origin) * inv;
    if t1 > t2 {
        std::mem::swap(&mut t1, &mut t2);
    }
    Some((t1, t2))
}

fn point_in_aabb(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

