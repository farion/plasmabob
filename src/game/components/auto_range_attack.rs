use bevy::prelude::{Component, Timer, TimerMode};
use std::time::Duration;

/// Autonomous ranged attack used by enemies or turrets.
#[derive(Component, Debug)]
pub struct AutoRangeAttack {
    /// Damage dealt per projectile.
    pub damage: i32,
    /// Range in virtual units.
    pub range: f32,
    /// Projectile speed in virtual units per second.
    pub speed: f32,
    /// Detection radius for selecting a target.
    pub aggro_range: f32,
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
            particle_effect: Some("plasma".to_string()),
            shoot_effect: Some("plasma_shoot".to_string()),
            impact_effect: Some("plasma_impact".to_string()),
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

impl AutoRangeAttack {
    /// Apply overrides from JSON object.
    ///
    /// Supported keys: damage, range, speed/projectile_speed, aggro_range,
    /// cooldown_ms/interval_s, particle_effect, shoot_effect, impact_effect, enabled.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(d) = map.get("damage").and_then(|n| n.as_i64()) {
                self.damage = d as i32;
            }
            if let Some(r) = map.get("range").and_then(|n| n.as_f64()) {
                self.range = r as f32;
            }
            if let Some(v) = map
                .get("projectile_speed")
                .or_else(|| map.get("speed"))
                .and_then(|n| n.as_f64())
            {
                self.speed = v as f32;
            }
            if let Some(v) = map.get("aggro_range").and_then(|n| n.as_f64()) {
                self.aggro_range = v as f32;
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
            if let Some(effect) = map.get("particle_effect").and_then(|v| v.as_str()) {
                let effect_name = effect.to_string();
                self.particle_effect = Some(effect_name.clone());
                // Hardcoded mapping: <effect> -> <effect>_shoot + <effect>_impact.
                self.shoot_effect = Some(format!("{}_shoot", effect_name));
                self.impact_effect = Some(format!("{}_impact", effect_name));
            }
            if let Some(shoot_effect) = map.get("shoot_effect").and_then(|v| v.as_str()) {
                self.shoot_effect = Some(shoot_effect.to_string());
            }
            if let Some(impact_effect) = map.get("impact_effect").and_then(|v| v.as_str()) {
                self.impact_effect = Some(impact_effect.to_string());
            }
            if let Some(b) = map.get("enabled").and_then(|n| n.as_bool()) {
                self.enabled = b;
            }
        }
        self
    }
}

