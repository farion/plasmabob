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

impl AutoRangeAttack {
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

