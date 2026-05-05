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

crate::impl_override_with_u32_default!(Health, crate::game::level::configs::health_config::HealthConfig,
    current => max,
    pick_u32 => [max, current],
    pick_bool => [despawn_on_death],
    pick_u64 => [despawn_delay_ms],
);
