use bevy::prelude::{Component, Timer, TimerMode};

/// Autonomous ranged attack used by enemies or turrets.
#[derive(Component, Debug)]
pub struct AutoRangeAttack {
    /// Damage dealt per projectile.
    pub damage: i32,
    /// Range in virtual units.
    pub range: f32,
    /// How often the attack fires.
    pub cooldown: Timer,
    /// Whether the attack is currently active.
    pub enabled: bool,
}

impl AutoRangeAttack {
    pub fn new(damage: i32, range: f32, interval_s: f32) -> Self {
        AutoRangeAttack { damage, range, cooldown: Timer::from_seconds(interval_s, TimerMode::Repeating), enabled: true }
    }
}

impl Default for AutoRangeAttack {
    fn default() -> Self {
        AutoRangeAttack::new(1, 200.0, 1.0)
    }
}

