use bevy::prelude::{Component, Entity, Timer, Vec2};

pub const PLASMA_Z: f32 = 10.0;
pub const PLASMA_BEAM_PARTICLE_COUNT: usize = 14;
pub const PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE: f32 = 5.0;
pub const PLASMA_BEAM_PARTICLE_WIGGLE_SPEED: f32 = 10.0;
pub const PLASMA_BEAM_VISUAL_HALF_HEIGHT: f32 = 6.0;

pub const PLASMA_IMPACT_PARTICLE_COUNT: usize = 22;
pub const PLASMA_IMPACT_LIFETIME_SECS: f32 = 0.24;
pub const PLASMA_IMPACT_MIN_SPEED: f32 = 95.0;
pub const PLASMA_IMPACT_MAX_SPEED: f32 = 280.0;

#[derive(Component, Debug, Clone)]
pub struct PlasmaBeam {
    pub origin: Vec2,
    pub direction: f32,
    pub current_length: f32,
    pub target_projectile: Option<Entity>,
    pub lifetime: Option<Timer>,
}

impl PlasmaBeam {
    pub fn new(origin: Vec2, direction: f32, target_projectile: Option<Entity>) -> Self {
        Self {
            origin,
            direction,
            current_length: 0.0,
            target_projectile,
            lifetime: None,
        }
    }
}
