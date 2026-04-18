use bevy::prelude::*;

/// Projectile component.
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    /// The entity that spawned/fired this projectile.
    pub owner: Entity,
    /// Damage this projectile applies when it hits a damageable target.
    pub damage: i32,
    /// Remaining travel range before despawn.
    pub remaining_range: f32,
    /// Name of the shoot visual effect.
    pub shoot_effect: Option<String>,
    /// Name of the impact visual effect.
    pub impact_effect: Option<String>,
}

impl Projectile {
    pub fn new(
        owner: Entity,
        damage: i32,
        range: f32,
        shoot_effect: Option<String>,
        impact_effect: Option<String>,
    ) -> Self {
        Self {
            owner,
            damage,
            remaining_range: range.max(0.0),
            shoot_effect,
            impact_effect,
        }
    }
}


