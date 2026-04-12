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

