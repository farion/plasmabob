use bevy::prelude::{Color, Component, Timer, Vec2};

#[derive(Component)]
pub(crate) struct PlasmaBeamParticle {
    pub(crate) normalized_distance: f32,
    pub(crate) lane: f32,
    pub(crate) phase: f32,
    pub(crate) layer_scale: f32,
}

#[derive(Component)]
pub(crate) struct PlasmaImpactParticle {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: Timer,
    pub(crate) start_size: f32,
}

#[derive(Component)]
pub(crate) struct ProjectileEffectParticle {
    // Velocity applied to the particle each step (game uses this field in gfx systems)
    pub(crate) velocity: Vec2,
    // Remaining lifetime timer
    pub(crate) lifetime: Timer,
    // Initial size used for scaling animations
    pub(crate) start_size: f32,
    // Base color of the particle
    pub(crate) base_color: Color,
}

/// Return a deterministic pseudo-random value in [0,1) from a seed.
pub(crate) fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}
