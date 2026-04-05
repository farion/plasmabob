use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub(crate) struct RangeAttack {
    pub(crate) damage: i32,
    pub(crate) speed: f32,
    pub(crate) frequency: f32,
}

impl RangeAttack {
    pub(crate) fn new(damage: i32, speed: f32, frequency: f32) -> Self {
        Self { damage, speed, frequency }
    }
}

