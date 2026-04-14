use bevy::prelude::Component;

/// Health component representing hit points for entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
    /// If true, the entity is removed after death according to `despawn_delay_ms` + fade-out.
    pub despawn_on_death: bool,
    /// Delay in milliseconds before fade-out starts after death.
    pub despawn_delay_ms: u64,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Health {
            current: max,
            max,
            despawn_on_death: false,
            despawn_delay_ms: 0,
        }
    }

    pub fn set_current(&mut self, v: i32) {
        self.current = v.clamp(0, self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    pub fn damage(&mut self, amount: i32) -> i32 {
        let prev = self.current;
        self.current = (self.current - amount).clamp(0, self.max);
        prev - self.current
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let prev = self.current;
        self.current = (self.current + amount).clamp(0, self.max);
        self.current - prev
    }
}

impl Default for Health {
    fn default() -> Self {
        Health::new(1)
    }
}

impl Health {
    /// Apply overrides from `components.health` JSON object. The JSON may
    /// contain these fields:
    /// - `health` (number): max HP (also sets current HP)
    /// - `despawn_on_death` (bool): whether this dead entity should be removed
    /// - `despawn_delay_ms` (number): delay before fade starts
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("health").and_then(|v| v.as_u64()) {
                let max = v as i32;
                self.max = max;
                self.current = max;
            }
            if let Some(v) = map.get("despawn_on_death").and_then(|v| v.as_bool()) {
                self.despawn_on_death = v;
            }
            if let Some(v) = map.get("despawn_delay_ms").and_then(|v| v.as_u64()) {
                self.despawn_delay_ms = v;
            }
        }
        self
    }
}

