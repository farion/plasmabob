use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::time::Duration;

use crate::game::runtime_components::sound_state::{SoundSeqStage, SoundState};
use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::SfxEntity;

/// Marker for loop-sound entities so we can identify and stop them on level exit.
#[derive(Component)]
pub struct StateSoundLoop;

/// Drives state-sound sequencing for every entity that has a [`SoundState`] component.
///
/// Detection: compares `SoundState::last_state` with the current `StateMachine` state.
/// When they differ a transition has occurred:
///   1. Stop the previous loop (despawn the loop entity).
///   2. Play the old state's `sound_end` (fire-and-forget).
///   3. If the new state has a `sound_start`: play it and start a countdown timer.
///      After the timer expires, spawn the `sound_loop` (if defined).
///   4. If the new state has no `sound_start` but has a `sound_loop`: spawn it immediately.
///
/// Per-frame:  tick the `WaitingForStartEnd` timer and spawn the loop when it fires.
pub fn sound_system(
    mut commands: Commands,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    entity_type_assets: Option<Res<EntityTypeAssets>>,
    mut query: Query<(
        &crate::game::components::StateMachine,
        &mut SoundState,
        Option<&SpawnedLevelEntity>,
    )>,
) {
    let vol = Volume::Linear(if audio_settings.sounds_enabled {
        audio_settings.sounds_volume
    } else {
        0.0
    });

    for (sm, mut ss, spawned) in &mut query {
        // ── Detect state transitions ──────────────────────────────────────
        if sm.state != ss.last_state {
            let _old_state = ss.last_state;
            let new_state = sm.state;

            // 1. Stop the running loop (if any).
            if let SoundSeqStage::Looping { loop_entity } = ss.stage {
                commands.entity(loop_entity).try_despawn();
            }

            // 2. Play the old state's end sound (fire-and-forget).
            if let Some(end_handle) = ss.end_handle.take() {
                commands.spawn((
                    AudioPlayer::<AudioSource>::new(end_handle),
                    PlaybackSettings {
                        mode: PlaybackMode::Once,
                        volume: vol,
                        ..default()
                    },
                    SfxEntity,
                ));
            }

            // 3. Look up new state's sounds from EntityTypeAssets.
            let (new_start, new_loop, new_end) =
                if let (Some(eta), Some(sel)) = (entity_type_assets.as_deref(), spawned) {
                    let state_name = new_state.to_state_name();
                    if let Some(sa) = eta.get_state(&sel.entity_type, state_name) {
                        (
                            sa.sound_start.clone(),
                            sa.sound_loop.clone(),
                            sa.sound_end.clone(),
                        )
                    } else {
                        (None, None, None)
                    }
                } else {
                    (None, None, None)
                };

            // Store end handle for when we leave this new state.
            ss.end_handle = new_end;

            // 4. Start new state's sounds.
            match new_start {
                Some((start_handle, duration_secs)) => {
                    // Play start sound.
                    commands.spawn((
                        AudioPlayer::<AudioSource>::new(start_handle),
                        PlaybackSettings {
                            mode: PlaybackMode::Once,
                            volume: vol,
                            ..default()
                        },
                        SfxEntity,
                    ));
                    // Set up a timer so we start the loop after start ends.
                    let timer = Timer::new(
                        Duration::from_secs_f32(duration_secs.max(0.0)),
                        TimerMode::Once,
                    );
                    ss.stage = SoundSeqStage::WaitingForStartEnd {
                        timer,
                        pending_loop: new_loop,
                    };
                }
                None => {
                    // No start sound → spawn loop immediately (if defined).
                    if let Some(loop_handle) = new_loop {
                        let loop_entity = commands
                            .spawn((
                                AudioPlayer::<AudioSource>::new(loop_handle),
                                PlaybackSettings {
                                    mode: PlaybackMode::Loop,
                                    volume: vol,
                                    ..default()
                                },
                                StateSoundLoop,
                            ))
                            .id();
                        ss.stage = SoundSeqStage::Looping { loop_entity };
                    } else {
                        ss.stage = SoundSeqStage::Idle;
                    }
                }
            }

            ss.last_state = new_state;
        }

        // ── Tick start-sound timer ────────────────────────────────────────
        if let SoundSeqStage::WaitingForStartEnd {
            ref mut timer,
            ref mut pending_loop,
        } = ss.stage
        {
            timer.tick(time.delta());
            if timer.just_finished() {
                if let Some(loop_handle) = pending_loop.take() {
                    let vol_loop = Volume::Linear(if audio_settings.sounds_enabled {
                        audio_settings.sounds_volume
                    } else {
                        0.0
                    });
                    let loop_entity = commands
                        .spawn((
                            AudioPlayer::<AudioSource>::new(loop_handle),
                            PlaybackSettings {
                                mode: PlaybackMode::Loop,
                                volume: vol_loop,
                                ..default()
                            },
                            StateSoundLoop,
                        ))
                        .id();
                    ss.stage = SoundSeqStage::Looping { loop_entity };
                } else {
                    ss.stage = SoundSeqStage::Idle;
                }
            }
        }
    }
}
