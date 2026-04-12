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

impl ControlledMeleeAttack {
    /// Apply overrides from JSON object. Keys: damage, range, cooldown_ms or cooldown_s.
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
            } else if let Some(s) = map.get("cooldown_s").and_then(|n| n.as_f64()) {
                self.cooldown = Timer::from_seconds(s as f32, TimerMode::Repeating);
            }
        }
        self
    }
}

