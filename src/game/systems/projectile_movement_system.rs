use bevy::prelude::*;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::RigidBody;
use crate::game::gfx::fire_shoot::{ensure_fire_particle_image, FireParticleImage, spawn_fire_shoot_particles};
use crate::game::gfx::plasma_shoot::{ensure_plasma_particle_image, PlasmaParticleImage};
use crate::game::gfx::poison::spawn_poison_particles;
use crate::game::gfx::spit::spawn_spit_particles;
use crate::game::runtime_components::Projectile;

const BEAM_AFTERGLOW_SECS: f32 = 0.28;

pub fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    mut fire_particle_image: Local<Option<Handle<Image>>>,
    particle_image_res: Option<Res<PlasmaParticleImage>>,
    fire_particle_image_res: Option<Res<FireParticleImage>>,
    mut trail_tick: Local<u64>,
    mut projectiles: Query<(Entity, &mut Transform, &RigidBody, &mut Projectile)>,
    mut beams: Query<(Entity, &mut PlasmaBeam)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    *trail_tick = trail_tick.wrapping_add(1);

    let mut expired_projectiles: Vec<(Entity, Vec2, f32)> = Vec::new();

    for (projectile_entity, mut transform, rigid_body, mut projectile) in &mut projectiles {
        let motion = rigid_body.velocity * dt;
        transform.translation.x += motion.x;
        transform.translation.y += motion.y;
        projectile.remaining_range = (projectile.remaining_range - motion.length()).max(0.0);

        // Keep non-plasma balls visible while they fly using particle-only trails.
        // Emit trails at a moderate rate to form a continuous plume without overdraw.
        if *trail_tick % 2 == 0 {
            let shoot_effect = projectile.shoot_effect.as_deref().unwrap_or("plasma_shoot");
            if is_ball_shoot_effect(shoot_effect) {
                // Fire uses its own dedicated particle image; other effects use the plasma image.
                let particle_image = if shoot_effect.eq_ignore_ascii_case("fire_shoot") {
                    if let Some(resource) = fire_particle_image_res.as_ref() {
                        resource.0.clone()
                    } else {
                        ensure_fire_particle_image(&mut fire_particle_image, &mut images)
                    }
                } else if let Some(resource) = particle_image_res.as_ref() {
                    resource.0.clone()
                } else {
                    ensure_plasma_particle_image(&mut plasma_particle_image, &mut images)
                };
                spawn_projectile_trail(
                    &mut commands,
                    &particle_image,
                    shoot_effect,
                    transform.translation.truncate(),
                    transform.translation.z,
                    rigid_body.velocity,
                    projectile_entity,
                    *trail_tick,
                );
            }
        }

        if projectile.remaining_range <= f32::EPSILON {
            let impact_position = transform.translation.truncate();
            let impact_z = transform.translation.z;
            // Do NOT spawn impact or sfx here — impact should only be created when a projectile
            // actually hits an entity (handled in projectile_collision_system). When the
            // projectile simply expires due to range, we only set beams to afterglow and despawn.
            expired_projectiles.push((projectile_entity, impact_position, impact_z));
        }
    }

    if expired_projectiles.is_empty() {
        return;
    }

    for (projectile_entity, _impact_position, _impact_z) in expired_projectiles {
        // Only set the beam to afterglow and despawn the projectile. Do not spawn impact
        // visuals or sounds for range expiration — impacts are only for real hits.
        set_beams_to_afterglow(projectile_entity, &mut beams);
        commands.entity(projectile_entity).despawn();
    }
}

fn is_ball_shoot_effect(effect: &str) -> bool {
    effect.eq_ignore_ascii_case("fire_shoot")
        || effect.eq_ignore_ascii_case("poison_shoot")
        || effect.eq_ignore_ascii_case("spit_shoot")
}

fn spawn_projectile_trail(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    shoot_effect: &str,
    position: Vec2,
    z: f32,
    velocity: Vec2,
    projectile_entity: Entity,
    trail_tick: u64,
) {
    let entity_seed = (projectile_entity.to_bits() & 0xFFFF_FFFF) as u32;
    let tick_seed = (trail_tick as u32).wrapping_mul(31_337);
    let seed = entity_seed ^ tick_seed;
    if shoot_effect.eq_ignore_ascii_case("fire_shoot") {
        let direction = velocity.normalize_or_zero();
        spawn_fire_shoot_particles(commands, particle_image, position, z, seed, direction);
    } else if shoot_effect.eq_ignore_ascii_case("poison_shoot") {
        spawn_poison_particles(commands, particle_image, position, z, seed, velocity * 0.25);
    } else if shoot_effect.eq_ignore_ascii_case("spit_shoot") {
        spawn_spit_particles(commands, particle_image, position, z, seed);
    }
}

fn set_beams_to_afterglow(projectile_entity: Entity, beams: &mut Query<(Entity, &mut PlasmaBeam)>) {
    for (_beam_entity, mut beam) in beams {
        if beam.target_projectile == Some(projectile_entity) {
            beam.target_projectile = None;
            beam.lifetime = Some(Timer::from_seconds(BEAM_AFTERGLOW_SECS, TimerMode::Once));
        }
    }
}

