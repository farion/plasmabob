use bevy::audio::{AudioPlayer, AudioSink, AudioSource, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::helper::active_character::ActiveCharacter;
use crate::helper::asset_io::load_character_asset;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::audio_toast::AudioToastRequest;
use crate::helper::key_bindings::{KeyAction, KeyBindings};
use crate::helper::music::MusicEntity;

fn effective_sounds_volume(audio_settings: &AudioSettings) -> f32 {
    if audio_settings.sounds_enabled {
        audio_settings.sounds_volume
    } else {
        0.0
    }
}

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
            .add_systems(Update, apply_sounds_volume_change_sinks)
            .add_systems(Update, apply_sounds_playback_settings_without_sink)
            .add_systems(Update, cleanup_finished_sfx);
    }
}

/// Preload frequently-used combat sound effects at startup so the first play
/// does not incur an on-demand load hitch. We deliberately only call
/// `asset_server.load()` here - the returned handles are dropped, but the
/// AssetServer will begin loading the audio into memory immediately.
fn preload_combat_sfx(asset_server: Res<AssetServer>, active_character: Res<ActiveCharacter>) {
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
            volume: Volume::Linear(effective_sounds_volume(audio_settings)),
            ..default()
        },
        SfxEntity,
        CombatSfx,
    ));
}

fn toggle_sound_mute(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut sinks: Query<&mut AudioSink, Without<MusicEntity>>,
    mut audio_settings: ResMut<AudioSettings>,
    mut toast_request: ResMut<AudioToastRequest>,
) {
    let toggle_key = key_bindings.get(KeyAction::ToggleSound);
    if !keys.as_ref().just_pressed(toggle_key) {
        return;
    }

    let new_enabled = !audio_settings.sounds_enabled;
    if audio_settings.set_sounds_enabled(new_enabled) {
        let _ = audio_settings.save_to_disk();
    }

    toast_request.set(if new_enabled {
        "toast.sound.on"
    } else {
        "toast.sound.off"
    });

    // Apply immediately to all non-music sinks (short SFX and loops).
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(effective_sounds_volume(&audio_settings)));
    }
}

fn apply_sounds_volume_change_sinks(
    audio_settings: Res<AudioSettings>,
    mut sinks: Query<&mut AudioSink, Without<MusicEntity>>,
) {
    if !audio_settings.is_changed() {
        return;
    }

    let sounds_volume = effective_sounds_volume(&audio_settings);
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(sounds_volume));
    }
}

fn apply_sounds_playback_settings_without_sink(
    audio_settings: Res<AudioSettings>,
    mut playbacks_without_sink: Query<&mut PlaybackSettings, (Without<AudioSink>, Without<MusicEntity>)>,
) {
    if !audio_settings.is_changed() {
        return;
    }

    let sounds_volume = effective_sounds_volume(&audio_settings);
    for mut playback in &mut playbacks_without_sink {
        playback.volume = Volume::Linear(sounds_volume);
    }
}

fn cleanup_finished_sfx(mut commands: Commands, sinks: Query<(Entity, &AudioSink), With<SfxEntity>>) {
    for (entity, sink) in &sinks {
        if sink.empty() {
            commands.entity(entity).despawn();
        }
    }
}

