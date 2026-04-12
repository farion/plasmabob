use bevy::prelude::*;

/// Projectile component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Projectile {
    /// The entity that spawned/fired this projectile.
    pub owner: Entity,
}

impl Projectile {
    pub fn new(owner: Entity) -> Self {
        Self { owner }
    }
}


