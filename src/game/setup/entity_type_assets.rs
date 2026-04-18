use bevy::audio::AudioSource;
use bevy::prelude::*;
use std::collections::HashMap;

/// Preloaded assets for a single animation state of an entity type.
#[derive(Debug, Clone)]
pub struct StateAssets {
    /// Preloaded image handles for the animation frames of this state.
    /// If empty, the idle/fallback state's frames are used instead (missing-sprite fallback = red quad).
    pub frames: Vec<Handle<Image>>,
    /// Duration of each frame in milliseconds.
    pub animation_frame_ms: u64,
    /// Minimum time (ms) to stay in this state before transitioning out.
    pub lock_ms: u64,
    /// Collision box in sprite-image space ([[x, y], ...]).
    pub collider_box: Option<Vec<[f32; 2]>>,
    /// Sound played once on state enter, with its duration in seconds (or None if unknown).
    pub sound_start: Option<(Handle<AudioSource>, f32)>,
    /// Sound looped while the state is active (starts after sound_start ends).
    pub sound_loop: Option<Handle<AudioSource>>,
    /// Sound played once on state exit.
    pub sound_end: Option<Handle<AudioSource>>,
}

/// All preloaded assets for a single entity type, keyed by state name (lowercase).
#[derive(Debug, Clone)]
pub struct EntityTypeAsset {
    /// State name → per-state assets.
    pub states: HashMap<String, StateAssets>,
    /// Name of the fallback state (typically the `initial_state`, e.g. "idle").
    pub fallback_state: String,
    /// Sprite width in world units (from entity type JSON).
    pub sprite_width: f32,
    /// Sprite height in world units (from entity type JSON).
    pub sprite_height: f32,
}

impl EntityTypeAsset {
    /// Look up state assets by name, falling back to the `fallback_state` when the
    /// requested state is not defined.
    pub fn get_state(&self, state_name: &str) -> Option<&StateAssets> {
        self.states
            .get(state_name)
            .or_else(|| self.states.get(&self.fallback_state))
    }
}

/// Resource holding deduplicated, preloaded assets for every entity type used in the current
/// level.  Built by the `LoadViewPlugin` before gameplay starts and removed when the level
/// exits so GPU/memory is freed between levels.
#[derive(Resource, Default, Debug)]
pub struct EntityTypeAssets {
    pub map: HashMap<String, EntityTypeAsset>,
}

impl EntityTypeAssets {
    /// Convenience accessor: look up state assets for `entity_type` / `state_name`.
    pub fn get_state(&self, entity_type: &str, state_name: &str) -> Option<&StateAssets> {
        self.map.get(entity_type)?.get_state(state_name)
    }
}

