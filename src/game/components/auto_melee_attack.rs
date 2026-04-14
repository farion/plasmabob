use bevy::prelude::{Component, Timer, TimerMode};
use std::time::Duration;

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
                let dur = Duration::from_millis(ms.max(1));
                let mut t = Timer::new(dur, TimerMode::Repeating);
                t.set_elapsed(dur);
                self.cooldown = t;
            } else if let Some(s) = map.get("interval_s").and_then(|n| n.as_f64()) {
                let dur = Duration::from_secs_f32((s as f32).max(f32::EPSILON));
                let mut t = Timer::new(dur, TimerMode::Repeating);
                t.set_elapsed(dur);
                self.cooldown = t;
            }
            if let Some(b) = map.get("enabled").and_then(|n| n.as_bool()) {
                self.enabled = b;
            }
        }
        self
    }
}

