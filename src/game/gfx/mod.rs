use bevy::prelude::*;

pub mod fire;
pub mod helpers;
pub mod poison;
pub mod spit;

/// Shared helper used by the three projectile effect modules.
/// Spawns `count` particles using the provided functions for velocity/size/color.
pub(crate) fn spawn_effect_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    count: usize,
    velocity_fn: impl Fn(u32) -> Vec2,
    size_fn: impl Fn(u32) -> f32,
    color_fn: impl Fn(u32) -> Color,
    lifetime_secs: f32,
) {
    for i in 0..count {
        let seed = seed_base.wrapping_add(i as u32).wrapping_mul(31337);
        let velocity = velocity_fn(seed);
        let size = size_fn(seed);
        let base_color = color_fn(seed);

        commands.spawn((
            Name::new("ProjectileEffectParticle"),
            Sprite {
                image: image.clone(),
                color: base_color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(origin.x, origin.y, z),
            crate::game::gfx::helpers::ProjectileEffectParticle {
                velocity,
                lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                start_size: size,
                base_color,
            },
            crate::game::runtime_components::GameEntity,
        ));
    }
}
