use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub(crate) struct Health {
    pub(crate) current: i32,
    pub(crate) max: i32,
}

impl Health {
    pub(crate) fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub(crate) fn is_dead(&self) -> bool {
        self.current <= 0
    }

    pub(crate) fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Damage(pub(crate) i32);

/// Tracks remaining invincibility frames after taking damage.
#[derive(Component, Debug, Clone)]
pub(crate) struct InvincibilityTimer(pub(crate) Timer);

impl InvincibilityTimer {
    pub(crate) fn new(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        timer.reset();
        Self(timer)
    }
}

/// Helper used by `spawn_entity` to insert a `Health` component with the
/// provided hp value.
pub(crate) fn insert(entity: &mut EntityCommands, hp: i32) {
    entity.insert(Health::new(hp));
}


