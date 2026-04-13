use bevy::prelude::{Component, Timer, TimerMode};

/// Range attack component for player-controlled ranged weapons (e.g. plasma gun).
#[derive(Component, Debug)]
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
}

impl ControlledRangeAttack {
    pub fn new(damage: i32, range: f32, speed: f32, cooldown_s: f32) -> Self {
        let mut cooldown = Timer::from_seconds(cooldown_s, TimerMode::Repeating);
        // Start ready so the player can fire immediately.
        cooldown.tick(std::time::Duration::from_secs_f32(cooldown_s.max(0.0)));
        ControlledRangeAttack {
            damage,
            range,
            speed,
            cooldown,
            projectile_type: None,
            shoot_effect: Some("plasma_shoot".to_string()),
            impact_effect: Some("plasma_impact".to_string()),
        }
    }
}

impl std::default::Default for ControlledRangeAttack {
    fn default() -> Self {
        ControlledRangeAttack::new(1, 200.0, 1200.0, 0.25)
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
            if let Some(v) = map
                .get("projectile_speed")
                .or_else(|| map.get("speed"))
                .and_then(|n| n.as_f64())
            {
                self.speed = v as f32;
            }
            if let Some(ms) = map.get("cooldown_ms").and_then(|n| n.as_u64()) {
                let cooldown_s = ms as f32 / 1000.0;
                self.cooldown = Timer::from_seconds(cooldown_s, TimerMode::Repeating);
                self.cooldown
                    .tick(std::time::Duration::from_secs_f32(cooldown_s.max(0.0)));
            } else if let Some(s) = map.get("cooldown_s").and_then(|n| n.as_f64()) {
                let cooldown_s = s as f32;
                self.cooldown = Timer::from_seconds(cooldown_s, TimerMode::Repeating);
                self.cooldown
                    .tick(std::time::Duration::from_secs_f32(cooldown_s.max(0.0)));
            }
            if let Some(projectile_type) = map.get("projectile_type").and_then(|v| v.as_str()) {
                self.projectile_type = Some(projectile_type.to_string());
            }
            if let Some(shoot_effect) = map.get("shoot_effect").and_then(|v| v.as_str()) {
                self.shoot_effect = Some(shoot_effect.to_string());
            }
            if let Some(impact_effect) = map.get("impact_effect").and_then(|v| v.as_str()) {
                self.impact_effect = Some(impact_effect.to_string());
            }
        }
        self
    }
}

