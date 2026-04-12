use bevy::prelude::{Component, Timer, TimerMode};

/// Simple autonomous melee attack (enemies that swipe or bite).
#[derive(Component, Debug, Clone)]
pub struct AutoMeleeAttack {
    /// Damage applied on hit.
    pub damage: i32,
    /// Range (radius) of the melee attack.
    pub range: f32,
    /// Cooldown timer between swings.
    pub cooldown: Timer,
    /// Enabled flag.
    pub enabled: bool,
}

impl AutoMeleeAttack {
    pub fn new(damage: i32, range: f32, interval_s: f32) -> Self {
        AutoMeleeAttack { damage, range, cooldown: Timer::from_seconds(interval_s, TimerMode::Repeating), enabled: true }
    }
}

impl Default for AutoMeleeAttack {
    fn default() -> Self {
        AutoMeleeAttack::new(1, 12.0, 0.5)
    }
}

