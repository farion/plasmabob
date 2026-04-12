use bevy::prelude::{Component, Timer, TimerMode};

/// Range attack component for player-controlled ranged weapons (e.g. plasma gun).
#[derive(Component, Debug)]
pub struct ControlledRangeAttack {
    /// Damage dealt per shot.
    pub damage: i32,
    /// Range in virtual units.
    pub range: f32,
    /// Cooldown timer between shots.
    pub cooldown: Timer,
}

impl ControlledRangeAttack {
    pub fn new(damage: i32, range: f32, cooldown_s: f32) -> Self {
        ControlledRangeAttack {
            damage,
            range,
            cooldown: Timer::from_seconds(cooldown_s, TimerMode::Repeating),
        }
    }
}

impl Default for ControlledRangeAttack {
    fn default() -> Self {
        ControlledRangeAttack::new(1, 200.0, 0.25)
    }
}

