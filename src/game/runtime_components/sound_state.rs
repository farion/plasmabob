use bevy::audio::AudioSource;
use bevy::prelude::*;

use crate::game::components::state_machine::EntityState;

/// Sequencing stage for per-entity state sounds.
#[derive(Debug, Clone)]
pub enum SoundSeqStage {
    /// No sound currently active for this entity.
    Idle,
    /// Waiting for the start sound to finish before launching the loop.
    /// The inner Timer counts down the start-sound duration.
    /// `pending_loop` is the loop handle to spawn when the timer fires.
    WaitingForStartEnd {
        timer: Timer,
        pending_loop: Option<Handle<AudioSource>>,
    },
    /// The loop sound is playing. `loop_entity` is the ECS entity for that sound.
    Looping { loop_entity: Entity },
}

/// Tracks the sound-sequencing state for one game entity across state transitions.
///
/// On every state transition the system:
/// 1. Stops any running loop (despawns `loop_entity`).
/// 2. Plays the old state's `sound_end` (fire-and-forget).
/// 3. Loads the new state's sound handles from `EntityTypeAssets`.
/// 4. Plays `sound_start`; when it ends, plays `sound_loop` (if defined).
#[derive(Component, Debug)]
pub struct SoundState {
    /// The entity state this `SoundState` was last synchronised to.
    /// Used by `sound_system` to detect transitions.
    pub last_state: EntityState,
    /// Current sequencing stage.
    pub stage: SoundSeqStage,
    /// End-sound handle of the *current* state, played when the entity leaves this state.
    pub end_handle: Option<Handle<AudioSource>>,
}

impl SoundState {
    /// Create a fresh `SoundState` for the given initial entity state (no sounds playing).
    pub fn new(initial_state: EntityState) -> Self {
        SoundState {
            last_state: initial_state,
            stage: SoundSeqStage::Idle,
            end_handle: None,
        }
    }
}

