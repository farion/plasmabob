use bevy::prelude::{Component, Timer, TimerMode};
use std::time::Duration;
// macro will reference helpers directly via crate::helper::override_helpers

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
    /// One-frame signal set by `auto_melee_attack_system` when an attack fires.
    /// Read and cleared by `state_machine_update_system` to trigger `MeleeAttacking`.
    pub just_attacked: bool,
}

impl AutoMeleeAttack {
    pub fn new(damage: i32, range: f32, interval_s: f32) -> Self {
        let duration = Duration::from_secs_f32(interval_s.max(f32::EPSILON));
        let mut cooldown = Timer::new(duration, TimerMode::Repeating);
        // Pre-elapse the timer so the first overlap fires damage immediately.
        cooldown.set_elapsed(duration);
        AutoMeleeAttack { damage, range, cooldown, enabled: true, just_attacked: false }
    }
}

impl Default for AutoMeleeAttack {
    fn default() -> Self {
        AutoMeleeAttack::new(1, 12.0, 0.5)
    }
}


crate::impl_override_from_config!(AutoMeleeAttack, crate::game::level::configs::AutoMeleeAttackConfig,
    pick_i32 => [damage],
    pick_f32 => [range],
    pick_bool => [enabled],
    pick_timer => [cooldown],
);
