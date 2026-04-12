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

impl ControlledRangeAttack {
    /// Apply overrides from JSON object. Accepts keys: damage (int), range (number), cooldown_ms (int) or cooldown_s (number).
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

