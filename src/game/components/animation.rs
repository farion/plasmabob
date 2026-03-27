use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntityState {
    Default,
    Walk,
    Jump,
    Fight,
    Hit,
    Die,
}

impl EntityState {
    pub(crate) fn animation_key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Fight => "fight",
            Self::Hit => "hit",
            Self::Die => "die",
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct AnimationState {
    pub(crate) current: EntityState,
    pub(crate) version: u64,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            current: EntityState::Default,
            version: 0,
        }
    }
}

impl AnimationState {
    pub(crate) fn set(&mut self, next: EntityState) -> bool {
        if self.current == next {
            return false;
        }

        self.current = next;
        self.version = self.version.saturating_add(1);
        true
    }
}

#[derive(Component, Debug, Clone)]
pub(crate) struct HitStateTimer {
    pub(crate) timer: Timer,
    pub(crate) applied_at_state_version: u64,
}

impl HitStateTimer {
    pub(crate) fn new(seconds: f32, applied_at_state_version: u64) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        timer.reset();
        Self {
            timer,
            applied_at_state_version,
        }
    }
}

pub(crate) const HIT_STATE_SECONDS: f32 = 1.0;

/// Prevents low-priority state updates from cancelling the temporary `hit` state.
pub(crate) fn can_set_state(
    state: &AnimationState,
    hit_timer: Option<&HitStateTimer>,
    next: EntityState,
) -> bool {
    if state.current == EntityState::Die {
        return next == EntityState::Die;
    }

    if state.current == EntityState::Hit && hit_timer.is_some() {
        return matches!(next, EntityState::Hit | EntityState::Die);
    }

    true
}
