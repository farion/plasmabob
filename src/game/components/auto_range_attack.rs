use bevy::prelude::{Component, Timer, TimerMode};
use std::time::Duration;

/// Autonomous ranged attack used by enemies or turrets.
#[derive(Component, Debug, Clone)]
pub struct AutoRangeAttack {
    /// Damage dealt per projectile.
    pub damage: i32,
    /// Range in virtual units.
    pub range: f32,
    /// Projectile speed in virtual units per second.
    pub speed: f32,
    /// Detection radius for selecting a target.
    pub aggro_range: f32,
    /// Preferred minimum distance for ranged combat.
    // Movement-related settings migrated into `AutoMovement` (min_engage_distance,
    // kiting_enabled, kiting_hp_threshold). Attack component now only stores
    // attack-specific settings.
    /// How often the attack fires.
    pub cooldown: Timer,
    /// Raw particle effect key from JSON (e.g. "fire", "poison", "spit").
    pub particle_effect: Option<String>,
    /// Name of the shoot visual effect.
    pub shoot_effect: Option<String>,
    /// Name of the impact visual effect.
    pub impact_effect: Option<String>,
    /// Whether the attack is currently active.
    pub enabled: bool,
    /// One-frame signal set by `auto_range_attack_system` when this entity fires.
    /// Read and cleared by `state_machine_update_system` to trigger `RangeAttacking`.
    pub just_fired: bool,
}

impl AutoRangeAttack {
    pub fn new(damage: i32, range: f32, interval_s: f32) -> Self {
        let duration = Duration::from_secs_f32(interval_s.max(f32::EPSILON));
        let mut cooldown = Timer::new(duration, TimerMode::Repeating);
        // Pre-elapse so entities can shoot immediately when a target enters aggro range.
        cooldown.set_elapsed(duration);
        AutoRangeAttack {
            damage,
            range,
            speed: 400.0,
            aggro_range: 300.0,
            cooldown,
            particle_effect: Some("fire".to_string()),
            shoot_effect: Some("fire_shoot".to_string()),
            impact_effect: Some("fire_impact".to_string()),
            enabled: true,
            just_fired: false,
        }
    }
}

impl Default for AutoRangeAttack {
    fn default() -> Self {
        AutoRangeAttack::new(1, 200.0, 1.0)
    }
}

// Generate a typed `override_from_config` impl using the helper macro.
// The macro maps config fields to component fields using the pick_* helpers.
crate::impl_override_from_config!(AutoRangeAttack, crate::game::level::configs::AutoRangeAttackConfig,
    pick_i32 => [damage],
    pick_f32 => [range, speed, aggro_range],
    pick_timer => [cooldown],
    pick_string => [particle_effect, shoot_effect, impact_effect],
    pick_bool => [enabled],
);
