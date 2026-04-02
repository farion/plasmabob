use std::collections::HashMap;

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

pub(crate) const DEFAULT_ANIMATION_FRAME_SECONDS: f32 = 0.5;

#[derive(Component, Debug, Clone)]
pub(crate) struct AnimationPlayback {
    pub(crate) state_version: u64,
    pub(crate) frame_index: usize,
    pub(crate) frame_elapsed: f32,
    pub(crate) frame_duration_secs: f32,
}

impl Default for AnimationPlayback {
    fn default() -> Self {
        Self::new(DEFAULT_ANIMATION_FRAME_SECONDS)
    }
}

impl AnimationPlayback {
    pub(crate) fn new(frame_duration_secs: f32) -> Self {
        Self {
            state_version: 0,
            frame_index: 0,
            frame_elapsed: 0.0,
            frame_duration_secs: frame_duration_secs.max(0.001),
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub(crate) struct PreloadedAnimations(pub(crate) HashMap<String, Vec<Handle<Image>>>);

impl PreloadedAnimations {
    pub(crate) fn from_paths(
        asset_server: &AssetServer,
        paths_by_state: &HashMap<String, Vec<String>>,
    ) -> Self {
        let mut handles = HashMap::with_capacity(paths_by_state.len());
        for (state, paths) in paths_by_state {
            let frames: Vec<Handle<Image>> = paths
                .iter()
                .filter(|path| !path.is_empty())
                .map(|path| asset_server.load(path.to_string()))
                .collect();
            handles.insert(state.clone(), frames);
        }
        Self(handles)
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

#[derive(Component, Debug, Clone)]
pub(crate) struct FightStateTimer {
    pub(crate) timer: Timer,
}

impl FightStateTimer {
    pub(crate) fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

pub(crate) const FIGHT_STATE_SECONDS: f32 = 0.6;

/// Prevents low-priority state updates from cancelling the temporary `hit` or `fight` state.
pub(crate) fn can_set_state(
    state: &AnimationState,
    hit_timer: Option<&HitStateTimer>,
    fight_timer: Option<&FightStateTimer>,
    next: EntityState,
) -> bool {
    if state.current == EntityState::Die {
        return next == EntityState::Die;
    }

    // When in Hit or Fight with an active timer, allow any transition except
    // `Walk`. This makes Hit and Fight able to overwrite each other (and be
    // overwritten by Jump, Default, Die, etc.), but prevents low-priority
    // Walk updates from cancelling temporary combat animations.
    if state.current == EntityState::Hit && hit_timer.is_some() {
        return next != EntityState::Walk;
    }

    if state.current == EntityState::Fight && fight_timer.is_some() {
        return next != EntityState::Walk;
    }

    true
}
