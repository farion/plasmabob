use bevy::prelude::*;
use bevy::audio::{AudioSink, PlaybackSettings, Volume};

use crate::helper::audio_settings::AudioSettings;
use crate::helper::key_bindings::{KeyAction, KeyBindings};

/// Generic marker for all short-lived sound effect entities (non-music)
#[derive(Component)]
pub(crate) struct SfxEntity;

/// Category markers so we can later adjust volumes per group
#[derive(Component)]
pub(crate) struct CombatSfx;

#[derive(Component)]
pub(crate) struct VoiceSfx;

#[derive(Component)]
pub(crate) struct EnvironmentalSfx;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_sound_mute)
            // split workload to avoid conflicting access patterns (Without<AudioSink>
            // conflicts with a mutable AudioSink query in the same system)
            .add_systems(Update, apply_sounds_volume_change_sinks)
            .add_systems(Update, apply_sounds_playback_settings_without_sink);
    }
}

fn toggle_sound_mute(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut sinks: Query<&mut AudioSink, With<SfxEntity>>,
) {
    let toggle_key = key_bindings.get(KeyAction::ToggleSound);
    if !keys.just_pressed(toggle_key) {
        return;
    }

    for mut sink in &mut sinks {
        sink.toggle_mute();
    }
}

fn apply_sounds_volume_change_sinks(
    audio_settings: Res<AudioSettings>,
    mut sinks: Query<(&mut AudioSink, Option<&CombatSfx>, Option<&VoiceSfx>, Option<&EnvironmentalSfx>), With<SfxEntity>>,
) {
    if !audio_settings.is_changed() {
        return;
    }

    for (mut sink, combat, voice, env) in &mut sinks {
        let vol = if voice.is_some() {
            Volume::Linear(audio_settings.sounds_volume)
        } else if combat.is_some() {
            Volume::Linear(audio_settings.sounds_volume)
        } else if env.is_some() {
            Volume::Linear(audio_settings.sounds_volume)
        } else {
            Volume::Linear(audio_settings.sounds_volume)
        };

        sink.set_volume(vol);
    }
}

fn apply_sounds_playback_settings_without_sink(
    audio_settings: Res<AudioSettings>,
    mut playbacks_without_sink: Query<
        (
            &mut PlaybackSettings,
            Option<&CombatSfx>,
            Option<&VoiceSfx>,
            Option<&EnvironmentalSfx>,
        ),
        (With<SfxEntity>, Without<AudioSink>),
    >,
) {
    if !audio_settings.is_changed() {
        return;
    }

    for (mut playback, combat, voice, env) in &mut playbacks_without_sink {
        playback.volume = if voice.is_some() {
            bevy::audio::Volume::Linear(audio_settings.sounds_volume)
        } else if combat.is_some() || env.is_some() {
            bevy::audio::Volume::Linear(audio_settings.sounds_volume)
        } else {
            bevy::audio::Volume::Linear(audio_settings.sounds_volume)
        };
    }
}
