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

impl AutoMeleeAttack {
    /// Apply overrides from JSON object. Keys: damage, range, cooldown_ms or interval_s, enabled.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(d) = map.get("damage").and_then(|n| n.as_i64()) {
                self.damage = d as i32;
            }
            if let Some(r) = map.get("range").and_then(|n| n.as_f64()) {
                self.range = r as f32;
            }
            if let Some(ms) = map.get("cooldown_ms").and_then(|n| n.as_u64()) {
                self.cooldown = Timer::from_seconds(ms as f32 / 1000.0, TimerMode::Repeating);
            } else if let Some(s) = map.get("interval_s").and_then(|n| n.as_f64()) {
                self.cooldown = Timer::from_seconds(s as f32, TimerMode::Repeating);
            }
            if let Some(b) = map.get("enabled").and_then(|n| n.as_bool()) {
                self.enabled = b;
            }
        }
        self
    }
}

