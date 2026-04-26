use bevy::prelude::*;
// EventReader/EventWriter avoided: use a simple resource queue (JumpParticlesQueue)

use crate::game::gfx::helpers::hash_to_unit;
use crate::game::gfx::particles::create_round_particle_image;
use crate::game::gfx::spawn_effect_particles;
use bevy::prelude::Resource;

/// Event emitted when a unit jumps or lands.
#[derive(Debug, Clone, Copy)]
pub enum JumpParticlesEvent {
    Jump {
        origin: Vec2,
        horizontal_dir: f32,
        seed_base: u32,
    },
    Land {
        origin: Vec2,
        horizontal_dir: f32,
        seed_base: u32,
    },
}

/// Simple resource queue for jump/land particle events. Using a resource
/// avoids coupling to Bevy's EventReader/EventWriter types and keeps the API
/// straightforward.
#[derive(Resource, Default)]
pub struct JumpParticlesQueue(pub Vec<JumpParticlesEvent>);

const JUMP_PARTICLE_TEXTURE_SIZE: u32 = 48;

/// Preloaded jump particle image resource.
#[derive(Resource)]
pub(crate) struct JumpParticleImage(pub Handle<Image>);

/// Preload the jump particle image into the asset store so it is ready for use.
pub(crate) fn preload_jump_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(create_round_particle_image(JUMP_PARTICLE_TEXTURE_SIZE));
    commands.insert_resource(JumpParticleImage(handle));
}

/// Remove the preloaded jump particle image when the level exits.
pub(crate) fn cleanup_jump_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    res: Option<Res<JumpParticleImage>>,
) {
    if let Some(res) = res {
        let handle = res.0.clone();
        images.remove(handle.id());
        commands.remove_resource::<JumpParticleImage>();
    }
}

/// Return an existing handle or create and cache the jump particle image on demand.
pub(crate) fn ensure_jump_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
    preloaded: Option<&JumpParticleImage>,
) -> Handle<Image> {
    if let Some(pre) = preloaded {
        return pre.0.clone();
    }
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }
    let handle = images.add(create_round_particle_image(JUMP_PARTICLE_TEXTURE_SIZE));
    *local_handle = Some(handle.clone());
    handle
}

/// System that listens for JumpParticlesEvent and spawns dust/soil particles
/// oriented by the horizontal direction. Uses a small round particle image
/// created on-demand and cached in a local Option<Handle<Image>>.
pub(crate) fn jump_particles_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut queue: ResMut<JumpParticlesQueue>,
    preloaded: Option<Res<JumpParticleImage>>,
    mut local_handle: Local<Option<Handle<Image>>>,
) {
    if queue.0.is_empty() {
        return;
    }

    // Prefer preloaded resource if present (registered by setup plugin). Otherwise
    // fall back to a local cached handle created on demand.
    let preloaded_ref = preloaded.as_deref();
    let image = ensure_jump_particle_image(&mut local_handle, &mut images, preloaded_ref);

    for ev in queue.0.drain(..) {
        match ev {
                            JumpParticlesEvent::Jump { origin, horizontal_dir, seed_base } => {
                // Jump: bias upward and slightly forward
                let z = 0.04_f32;
                                let seed = seed_base;
                // Use spawn_effect_particles helper to spawn multiple particles
                // Bigger, denser jump puff: more particles, larger sizes and longer lifetime.
                spawn_effect_particles(
                    &mut commands,
                    &image,
                    origin,
                    z,
                    seed,
                    20,
                    |s| {
                        // velocity: stronger upward bias and more horizontal spread
                        let hx = horizontal_dir.signum() * (12.0 + hash_to_unit(s.wrapping_mul(11)) * 18.0);
                        let jitter_x = (hash_to_unit(s.wrapping_mul(13)) - 0.5) * 14.0;
                        let vy = 42.0 + hash_to_unit(s.wrapping_mul(17)) * 36.0;
                        Vec2::new(hx + jitter_x, vy)
                    },
                    |s| 12.0 + hash_to_unit(s.wrapping_mul(19)) * 16.0,
                    |s| {
                        // dusty brownish colors, slightly desaturated for larger puffs
                        let v = 0.20 + hash_to_unit(s.wrapping_mul(23)) * 0.22;
                        Color::srgba(0.44 + v * 0.26, 0.36 + v * 0.24, 0.26 + v * 0.16, 0.92)
                    },
                    0.36,
                );
            }
                            JumpParticlesEvent::Land { origin, horizontal_dir, seed_base } => {
                // Land: flatter outward burst, slightly upward to show dust
                let z = 0.02_f32;
                                let seed = seed_base;
                                                // Larger, heavier landing plume: more particles, broader spread, slightly longer lifetime
                                                spawn_effect_particles(
                                                    &mut commands,
                                                    &image,
                                                    origin,
                                                    z,
                                                    seed.wrapping_add(0xA5A5_A5A5),
                                                    28,
                                                    |s| {
                                                        let angle = hash_to_unit(s.wrapping_mul(7)) * std::f32::consts::TAU;
                                                        let radial = 18.0 + hash_to_unit(s.wrapping_mul(11)) * 48.0;
                                                        // bias slightly in facing direction
                                                        let bias = horizontal_dir.clamp(-1.0, 1.0) * 12.0;
                                                        Vec2::new(angle.cos() * radial + bias, angle.sin() * (radial * 0.55) + 10.0)
                                                    },
                                                    |s| 12.0 + hash_to_unit(s.wrapping_mul(13)) * 24.0,
                                                    |s| {
                                                        let v = 0.16 + hash_to_unit(s.wrapping_mul(17)) * 0.26;
                                                        Color::srgba(0.36 + v * 0.30, 0.30 + v * 0.24, 0.24 + v * 0.18, 0.94)
                                                    },
                                                    0.42,
                                                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    #[test]
    fn event_construction() {
        let _ = JumpParticlesEvent::Jump { origin: Vec2::ZERO, horizontal_dir: 1.0, seed_base: 0 };
        let _ = JumpParticlesEvent::Land { origin: Vec2::ZERO, horizontal_dir: -1.0, seed_base: 42 };
    }
}





