use bevy::prelude::{Component, Timer, TimerMode};

/// Melee attack component for player-controlled melee weapons.
#[derive(Component, Debug, Clone)]
pub struct ControlledMeleeAttack {
    /// Damage applied on hit.
    pub damage: i32,
    /// Range (radius) of the melee attack.
    pub range: f32,
    /// Cooldown between swings.
    pub cooldown: Timer,
}

impl ControlledMeleeAttack {
    pub fn new(damage: i32, range: f32, cooldown_s: f32) -> Self {
        ControlledMeleeAttack { damage, range, cooldown: Timer::from_seconds(cooldown_s, TimerMode::Repeating) }
    }
}

impl Default for ControlledMeleeAttack {
    fn default() -> Self {
        ControlledMeleeAttack::new(1, 12.0, 0.3)
    }
}

// Use the helper macro to implement `override_from_config` for simple fields
crate::impl_override_from_config!(ControlledMeleeAttack, crate::game::level::configs::ControlledMeleeAttackConfig,
    pick_i32 => [damage],
    pick_f32 => [range],
    pick_timer => [cooldown],
);

