use bevy::prelude::Component;

/// Health component representing hit points for entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Health { current: max, max }
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
    /// contain a `health` numeric field (max HP). If provided, both max and
    /// current are set to that value.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        let max = comp_obj
            .and_then(|o| o.get("health").and_then(|v| v.as_u64()))
            .map(|v| v as i32)
            .unwrap_or(self.max);
        self.max = max;
        self.current = max;
        self
    }
}

