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
    /// Apply overrides from JSON for Damageable.
    ///
    /// Supported keys:
    /// - `damaged_duration_secs`: number — how long the Damaged state lasts after a hit.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("damaged_duration_secs").and_then(|n| n.as_f64()) {
                self.damaged_duration_secs = (v as f32).max(0.0);
            }
        }
        self
    }
}
