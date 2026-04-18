use bevy::prelude::Component;

/// Damageable component indicates that an entity can take damage and potentially be destroyed.
#[derive(Component, Debug, Clone, Copy)]
pub struct Damageable {
    /// Duration in seconds the entity stays in the Damaged state after being hit.
    /// Can be overridden via JSON key `damaged_duration_secs`.
    pub damaged_duration_secs: f32,
    /// Remaining time in seconds of the current damaged state (counts down to 0 each frame).
    /// Set to `damaged_duration_secs` by the projectile collision system when damage is applied.
    pub damaged_timer: f32,
}

impl Damageable {
    pub fn new() -> Self {
        Damageable {
            damaged_duration_secs: 0.5,
            damaged_timer: 0.0,
        }
    }
}

impl Default for Damageable {
    fn default() -> Self {
        Damageable::new()
    }
}

impl Damageable {
}

// Use macro-based implementation for consistency with other components.
crate::impl_override_from_config!(Damageable, crate::game::level::configs::DamageableConfig,
    pick_f32 => [damaged_duration_secs],
);

