use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::{Blocking, Collider, ColliderShape, Damageable, Health, RigidBody, StateMachine, Team};
use crate::game::systems::damage_popup_system::spawn_damage_popup;
use crate::game::runtime_components::DamagePopupSettings;
use crate::game::gfx::fire_impact::spawn_fire_impact_explosion;
use crate::game::gfx::fire_shoot::{ensure_fire_particle_image, FireParticleImage};
use crate::game::gfx::plasma_impact::spawn_plasma_impact_explosion;
use crate::game::gfx::plasma_shoot::{ensure_plasma_particle_image, PlasmaParticleImage};
use crate::game::gfx::poison::spawn_poison_particles;
use crate::game::gfx::spit::spawn_spit_particles;
use crate::game::runtime_components::Projectile;
use crate::helper::active_character::ActiveCharacter;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::spawn_combat_sfx;

const TOI_EPSILON: f32 = 1e-4;
const NEUTRAL_TEAM: &str = "Neutral";
const PLASMA_HIT_SFX: &str = "audio/plasma-hit.ogg";
const BEAM_AFTERGLOW_SECS: f32 = 0.28;

#[derive(SystemParam)]
pub(crate) struct ProjectileCollisionRuntime<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    asset_server: Res<'w, AssetServer>,
    active_character: Res<'w, ActiveCharacter>,
    audio_settings: Res<'w, AudioSettings>,
    images: ResMut<'w, Assets<Image>>,
    plasma_particle_image: Local<'s, Option<Handle<Image>>>,
    fire_particle_image: Local<'s, Option<Handle<Image>>>,
    particle_image_res: Option<Res<'w, PlasmaParticleImage>>,
    fire_particle_image_res: Option<Res<'w, FireParticleImage>>,
    stats: ResMut<'w, crate::LevelStats>,
}

pub fn projectile_collision_system(
    mut runtime: ProjectileCollisionRuntime,
    projectiles: Query<(Entity, &Transform, &Collider, &RigidBody, &Projectile)>,
    targets: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&Blocking>,
            Option<&RigidBody>,
            Option<&Team>,
            Option<&crate::game::tags::PlayerTag>,
        ),
    >,
    teams: Query<&Team>,
    state_machines: Query<&StateMachine>,
    mut health_query: Query<&mut Health>,
    mut damageable_query: Query<&mut Damageable>,
    mut beams: Query<(Entity, &mut PlasmaBeam)>,
) {
    let dt = runtime.time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (projectile_entity, projectile_transform, projectile_collider, projectile_body, projectile) in
        &projectiles
    {
        if let Ok(owner_sm) = state_machines.get(projectile.owner) {
            if owner_sm.is_non_interactive() {
                set_beams_to_afterglow(projectile_entity, &mut beams);
                // Use try_despawn to silently ignore duplicate despawn attempts.
                runtime.commands.entity(projectile_entity).try_despawn();
                continue;
            }
        }

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

        for (target_entity, target_transform, target_collider, blocking, target_body, target_team, target_player_tag) in
            &targets
        {
            if target_entity == projectile.owner || target_entity == projectile_entity {
                continue;
            }

            let is_blocking = blocking.is_some();
            let is_damageable = damageable_query.contains(target_entity);
            if let Ok(target_sm) = state_machines.get(target_entity) {
                if target_sm.is_non_interactive() {
                    continue;
                }
            }
            if !is_blocking && !is_damageable {
                continue;
            }

            let target_team_name = target_team
                .map(|team| team.name.as_str())
                .unwrap_or(NEUTRAL_TEAM);
            if is_damageable && owner_team == target_team_name {
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
                impact_z: target_transform.translation.z,
                is_controlled: target_player_tag.is_some(),
            };

            if is_better_hit(candidate, best_hit) {
                best_hit = Some(candidate);
            }
        }

        if let Some(hit) = best_hit {
            if hit.is_damageable {
                if let Ok(mut health) = health_query.get_mut(hit.entity) {
                    let dealt = health.damage(projectile.damage);
                    if dealt > 0 {
                        runtime.stats.hits = runtime.stats.hits.saturating_add(1);
                    }
                }
                // Trigger the Damaged state by resetting the damaged timer.
                if let Ok(mut dmg) = damageable_query.get_mut(hit.entity) {
                    dmg.damaged_timer = dmg.damaged_duration_secs;
                }

                // Spawn floating damage numbers at the impact position.
                let pos = Vec3::new(hit.impact_position.x, hit.impact_position.y, hit.impact_z + 20.0);
                spawn_damage_popup(&mut runtime.commands, pos, projectile.damage as i32, false, hit.is_controlled, &DamagePopupSettings::default());
            }

            let impact_effect = projectile.impact_effect.as_deref().unwrap_or("plasma_impact");
            if is_supported_impact_effect(impact_effect) {
                // Fire uses its own dedicated particle image; all others share the plasma image.
                let particle_image = if impact_effect.eq_ignore_ascii_case("fire_impact") {
                    if let Some(resource) = runtime.fire_particle_image_res.as_ref() {
                        resource.0.clone()
                    } else {
                        ensure_fire_particle_image(&mut runtime.fire_particle_image, &mut runtime.images)
                    }
                } else if let Some(resource) = runtime.particle_image_res.as_ref() {
                    resource.0.clone()
                } else {
                    ensure_plasma_particle_image(&mut runtime.plasma_particle_image, &mut runtime.images)
                };
                let impact_z = projectile_transform.translation.z;
                spawn_projectile_impact_effect(
                    &mut runtime.commands,
                    &particle_image,
                    impact_effect,
                    hit.impact_position,
                    impact_z,
                    projectile_body.velocity,
                    entity_seed(projectile_entity),
                );
            }
            spawn_combat_sfx(
                &mut runtime.commands,
                &runtime.asset_server,
                &runtime.audio_settings,
                *runtime.active_character,
                PLASMA_HIT_SFX,
            );
            set_beams_to_afterglow(projectile_entity, &mut beams);

            // Use try_despawn to avoid warnings if the projectile was already scheduled
            // for despawn by another system earlier in the same frame.
            runtime.commands.entity(projectile_entity).try_despawn();
        }
    }
}

fn is_supported_impact_effect(effect: &str) -> bool {
    effect.eq_ignore_ascii_case("plasma_impact")
        || effect.eq_ignore_ascii_case("fire_impact")
        || effect.eq_ignore_ascii_case("poison_impact")
        || effect.eq_ignore_ascii_case("spit_impact")
}

fn spawn_projectile_impact_effect(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    impact_effect: &str,
    position: Vec2,
    z: f32,
    projectile_velocity: Vec2,
    seed_base: u32,
) {
    if impact_effect.eq_ignore_ascii_case("plasma_impact") {
        spawn_plasma_impact_explosion(commands, particle_image, position, z);
    } else if impact_effect.eq_ignore_ascii_case("fire_impact") {
        spawn_fire_impact_explosion(commands, particle_image, position, z);
    } else if impact_effect.eq_ignore_ascii_case("poison_impact") {
        spawn_poison_particles(
            commands,
            particle_image,
            position,
            z,
            seed_base,
            projectile_velocity * 0.2,
        );
    } else if impact_effect.eq_ignore_ascii_case("spit_impact") {
        spawn_spit_particles(commands, particle_image, position, z, seed_base);
    }
}

fn entity_seed(entity: Entity) -> u32 {
    (entity.to_bits() & 0xFFFF_FFFF) as u32
}

#[derive(Debug, Clone, Copy)]
struct HitCandidate {
    entity: Entity,
    toi: f32,
    distance: f32,
    class_rank: u8,
    is_damageable: bool,
    impact_position: Vec2,
    impact_z: f32,
    is_controlled: bool,
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

