use bevy::prelude::{Component, Entity, Timer, Vec2};

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
