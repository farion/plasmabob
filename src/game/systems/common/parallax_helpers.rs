use bevy::prelude::*;

pub(crate) const PARALLAX_BACKGROUND_SPEED: f32 = 0.08;
pub(crate) const PARALLAX_MIN_SPEED: f32 = 0.12;
pub(crate) const PARALLAX_MAX_SPEED: f32 = 1.5;
pub(crate) const PARALLAX_MIN_Z: f32 = 0.0;
pub(crate) const PARALLAX_MAX_Z: f32 = 150.0;
// Only apply parallax for entities outside the "no-parallax" middle band.
pub(crate) const PARALLAX_NO_EFFECT_LOWER_Z: f32 = 75.0;
pub(crate) const PARALLAX_NO_EFFECT_UPPER_Z: f32 = 125.0;

pub(crate) fn parallax_world_x(base_x: f32, camera_x: f32, speed: f32) -> f32 {
    base_x + camera_x * (1.0 - speed)
}

pub(crate) fn parallax_speed_from_z(z_index: f32) -> f32 {
    let normalized = ((z_index - PARALLAX_MIN_Z) / (PARALLAX_MAX_Z - PARALLAX_MIN_Z)).clamp(0.0, 1.0);
    PARALLAX_MIN_SPEED + normalized * (PARALLAX_MAX_SPEED - PARALLAX_MIN_SPEED)
}

