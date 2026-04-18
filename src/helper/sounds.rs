use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSink, PlaybackMode, PlaybackSettings, Volume, AudioSource};

use crate::helper::active_character::ActiveCharacter;
use crate::helper::asset_io::load_character_asset;
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
        app.add_systems(Startup, preload_combat_sfx)
            .add_systems(Update, toggle_sound_mute)
            // split workload to avoid conflicting access patterns (Without<AudioSink>
            // conflicts with a mutable AudioSink query in the same system)
            .add_systems(Update, apply_sounds_volume_change_sinks)
            .add_systems(Update, apply_sounds_playback_settings_without_sink)
            .add_systems(Update, cleanup_finished_sfx);
    }
}

/// Preload frequently-used combat sound effects at startup so the first play
/// does not incur an on-demand load hitch. We deliberately only call
/// `asset_server.load()` here — the returned handles are dropped, but the
/// AssetServer will begin loading the audio into memory immediately.
fn preload_combat_sfx(asset_server: Res<AssetServer>, active_character: Res<ActiveCharacter>) {
    // These are the short SFX the game plays frequently; preloading them
    // eliminates the audible delay the first time they're played.
    let _ = load_character_asset::<AudioSource>(&asset_server, "audio/plasma-shot.ogg", *active_character);
    let _ = load_character_asset::<AudioSource>(&asset_server, "audio/plasma-hit.ogg", *active_character);
    let _ = load_character_asset::<AudioSource>(&asset_server, "audio/cockroach-die.ogg", *active_character);
}

pub(crate) fn spawn_combat_sfx(
    commands: &mut Commands,
    asset_server: &AssetServer,
    audio_settings: &AudioSettings,
    active_character: ActiveCharacter,
    path: &'static str,
) {
    commands.spawn((
        AudioPlayer::new(load_character_asset::<AudioSource>(asset_server, path, active_character)),
        PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::Linear(audio_settings.sounds_volume),
            ..default()
        },
        SfxEntity,
        CombatSfx,
    ));
}

fn toggle_sound_mute(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut sinks: Query<&mut AudioSink, With<SfxEntity>>,
) {
    let toggle_key = key_bindings.get(KeyAction::ToggleSound);
    if !keys.as_ref().just_pressed(toggle_key) {
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
    let sounds_volume = audio_settings.as_ref().sounds_volume;

    for (mut sink, combat, voice, env) in &mut sinks {
        let vol = if voice.is_some() {
            Volume::Linear(sounds_volume)
        } else if combat.is_some() {
            Volume::Linear(sounds_volume)
        } else if env.is_some() {
            Volume::Linear(sounds_volume)
        } else {
            Volume::Linear(sounds_volume)
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
    let sounds_volume = audio_settings.as_ref().sounds_volume;

    for (mut playback, combat, voice, env) in &mut playbacks_without_sink {
        playback.volume = if voice.is_some() {
            bevy::audio::Volume::Linear(sounds_volume)
        } else if combat.is_some() || env.is_some() {
            bevy::audio::Volume::Linear(sounds_volume)
        } else {
            bevy::audio::Volume::Linear(sounds_volume)
        };
    }
}

fn cleanup_finished_sfx(mut commands: Commands, sinks: Query<(Entity, &AudioSink), With<SfxEntity>>) {
    for (entity, sink) in &sinks {
        if sink.empty() {
            commands.entity(entity).despawn();
        }
    }
}

