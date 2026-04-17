use bevy::prelude::{Component, Timer, TimerMode};

/// Range attack component for player-controlled ranged weapons (e.g. plasma gun).
#[derive(Component, Debug, Clone)]
pub struct ControlledRangeAttack {
    /// Damage dealt per shot.
    pub damage: i32,
    /// Range in virtual units.
    pub range: f32,
    /// Projectile speed in virtual units per second.
    pub speed: f32,
    /// Cooldown timer between shots.
    pub cooldown: Timer,
    /// Optional projectile type identifier for future data-driven projectile spawning.
    pub projectile_type: Option<String>,
    /// Name of the shoot visual effect.
    pub shoot_effect: Option<String>,
    /// Name of the impact visual effect.
    pub impact_effect: Option<String>,
    /// Set to `true` for exactly one frame when this entity fires a projectile.
    /// Cleared by `state_machine_update_system` after it reads the signal.
    pub just_fired: bool,
}

impl ControlledRangeAttack {
    pub fn new(damage: i32, range: f32, speed: f32, cooldown_s: f32) -> Self {
        let cooldown_secs = cooldown_s.max(0.0);
        // Use a one-shot timer: ready state stays true after finishing until reset on fire.
        let mut cooldown = Timer::from_seconds(cooldown_secs, TimerMode::Once);
        // Start ready so the very first trigger press fires immediately.
        cooldown.tick(std::time::Duration::from_secs_f32(cooldown_secs));
        ControlledRangeAttack {
            damage,
            range,
            speed,
            cooldown,
            projectile_type: None,
            shoot_effect: Some("plasma_shoot".to_string()),
            impact_effect: Some("plasma_impact".to_string()),
            just_fired: false,
        }
    }
}

impl std::default::Default for ControlledRangeAttack {
    fn default() -> Self {
        ControlledRangeAttack::new(1, 200.0, 1200.0, 0.25)
    }
}


// Use the macro to implement overrides; use pick_timer_once so cooldown is Once and ready.
crate::impl_override_from_config!(ControlledRangeAttack, crate::game::level::configs::ControlledRangeAttackConfig,
    pick_i32 => [damage],
    pick_f32 => [range, speed],
    pick_timer_once => [cooldown],
    pick_string => [projectile_type, shoot_effect, impact_effect],
);

