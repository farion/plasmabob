use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use avian2d::prelude::{Collider as AvCollider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};

use crate::game::components::plasma::PlasmaBeam;
use crate::game::debug_stats::DebugStats;
use crate::game::components::{
    Blocking, Collider, ColliderShape, Damageable, Health, RigidBody, StateMachine, Team,
};
use crate::game::gfx::fire_impact::spawn_fire_impact_explosion;
use crate::game::gfx::fire_shoot::{ensure_fire_particle_image, FireParticleImage};
use crate::game::gfx::plasma_impact::spawn_plasma_impact_explosion;
use crate::game::gfx::plasma_shoot::{ensure_plasma_particle_image, PlasmaParticleImage};
use crate::game::gfx::poison::spawn_poison_particles;
use crate::game::gfx::spit::spawn_spit_particles;
use crate::game::runtime_components::DamagePopupSettings;
use crate::game::runtime_components::Projectile;
use crate::game::systems::damage_popup_system::spawn_damage_popup;
use crate::helper::active_character::ActiveCharacter;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::spawn_combat_sfx;

const TOI_EPSILON: f32 = 1e-4;
const NEUTRAL_TEAM: &str = "Neutral";
const PLASMA_HIT_SFX: &str = "audio/plasma-hit.ogg";
const BEAM_AFTERGLOW_SECS: f32 = 0.28;
const PROJECTILE_SHAPE_MAX_HITS: u32 = 32;

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
    debug_stats: ResMut<'w, DebugStats>,
}

pub fn projectile_collision_system(
    mut runtime: ProjectileCollisionRuntime,
    spatial_query: SpatialQuery,
    projectiles: Query<(Entity, &Transform, &Collider, &RigidBody, &Projectile)>,
    targets: Query<(
        Entity,
        &Transform,
        &Collider,
        Option<&Blocking>,
        Option<&RigidBody>,
        Option<&Team>,
        Option<&crate::game::tags::PlayerTag>,
    )>,
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

    for (
        projectile_entity,
        projectile_transform,
        projectile_collider,
        projectile_body,
        projectile,
    ) in &projectiles
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

        let projectile_center =
            projectile_transform.translation.truncate() + projectile_collider.offset;
        let projectile_delta = projectile_body.velocity * dt;

        let Ok(cast_direction) = bevy::math::Dir2::new(projectile_delta) else {
            continue;
        };
        let cast_distance = projectile_delta.length();
        if cast_distance <= f32::EPSILON {
            continue;
        }

        runtime.debug_stats.projectile_shape_hits_calls = runtime
            .debug_stats
            .projectile_shape_hits_calls
            .saturating_add(1);

        let shape_config = ShapeCastConfig::from_max_distance(cast_distance);
        let shape = AvCollider::rectangle(
            projectile_half_extents.x * 2.0,
            projectile_half_extents.y * 2.0,
        );
        let filter = SpatialQueryFilter::from_excluded_entities([projectile_entity, projectile.owner]);
        let hits = spatial_query.shape_hits(
            &shape,
            projectile_center,
            0.0,
            cast_direction,
            PROJECTILE_SHAPE_MAX_HITS,
            &shape_config,
            &filter,
        );

        let mut best_hit: Option<HitCandidate> = None;

        for hit_data in hits {
            runtime.debug_stats.projectile_shape_hit_candidates = runtime
                .debug_stats
                .projectile_shape_hit_candidates
                .saturating_add(1);

            let Ok((
                target_entity,
                target_transform,
                _target_collider,
                blocking,
                _target_body,
                target_team,
                target_player_tag,
            )) = targets.get(hit_data.entity)
            else {
                continue;
            };

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

            let distance = hit_data.distance;
            let toi = if cast_distance > f32::EPSILON {
                (distance / cast_distance).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let class_rank = if is_blocking { 0_u8 } else { 1_u8 };
            let candidate = HitCandidate {
                entity: target_entity,
                toi,
                distance,
                class_rank,
                is_damageable,
                impact_position: hit_data.point1,
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
                let pos = Vec3::new(
                    hit.impact_position.x,
                    hit.impact_position.y,
                    hit.impact_z + 20.0,
                );
                spawn_damage_popup(
                    &mut runtime.commands,
                    pos,
                    projectile.damage as i32,
                    false,
                    hit.is_controlled,
                    &DamagePopupSettings::default(),
                );
            }

            let impact_effect = projectile
                .impact_effect
                .as_deref()
                .unwrap_or("plasma_impact");
            if is_supported_impact_effect(impact_effect) {
                // Fire uses its own dedicated particle image; all others share the plasma image.
                let particle_image = if impact_effect.eq_ignore_ascii_case("fire_impact") {
                    if let Some(resource) = runtime.fire_particle_image_res.as_ref() {
                        resource.0.clone()
                    } else {
                        ensure_fire_particle_image(
                            &mut runtime.fire_particle_image,
                            &mut runtime.images,
                        )
                    }
                } else if let Some(resource) = runtime.particle_image_res.as_ref() {
                    resource.0.clone()
                } else {
                    ensure_plasma_particle_image(
                        &mut runtime.plasma_particle_image,
                        &mut runtime.images,
                    )
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
