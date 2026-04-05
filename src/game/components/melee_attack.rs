use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct MeleeAttack {
    pub(crate) damage: i32,
}

impl MeleeAttack {
    pub(crate) fn new(damage: i32) -> Self {
        Self { damage }
    }
}

