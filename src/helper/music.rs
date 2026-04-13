use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume};

use crate::helper::audio_settings::AudioSettings;
use crate::helper::active_character::ActiveCharacter;
use crate::helper::key_bindings::{KeyAction, KeyBindings};

/// Pending request for the music player.
/// None == no request pending.
/// Some(Vec<String>) == play the given playlist (can be a single entry).
#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct MusicRequest(pub(crate) Option<Vec<String>>);

/// Marker for our central music entity
#[derive(Component)]
pub(crate) struct MusicEntity;

/// Keeps track of currently spawned music entity and its track path
#[derive(Resource, Default)]
pub(crate) struct MusicManager {
    pub(crate) entity: Option<Entity>,
    pub(crate) current_track: Option<String>,
    /// Optional playlist active while in a level
    pub(crate) playlist: Option<Vec<String>>,
    pub(crate) playlist_index: usize,
}

pub(crate) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicManager>()
            .init_resource::<MusicRequest>()
            .add_systems(Startup, start_background_music)
            .add_systems(Update, handle_music_requests)
            .add_systems(Update, advance_playlist_if_finished)
            .add_systems(Update, toggle_music_mute)
            .add_systems(Update, sync_music_track)
            .add_systems(Update, apply_music_volume_change);
    }
}

fn toggle_music_mute(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut sinks: Query<&mut AudioSink, With<MusicEntity>>,
) {
    let toggle_key = key_bindings.get(KeyAction::ToggleMute);
    if !keys.just_pressed(toggle_key) {
        return;
    }

    for mut sink in &mut sinks {
        sink.toggle_mute();
    }
}

fn start_background_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    active_character: Res<ActiveCharacter>,
    mut manager: ResMut<MusicManager>,
) {
    // spawn once at startup
    if manager.entity.is_some() {
        return;
    }

    let handle = asset_server.load(active_character.menu_music_path());
    let entity = commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: Volume::Linear(audio_settings.music_volume),
                ..default()
            },
            MusicEntity,
        ))
        .id();

    manager.entity = Some(entity);
    manager.current_track = Some(active_character.menu_music_path().to_string());
}

fn sync_music_track(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    active_character: Res<ActiveCharacter>,
    mut manager: ResMut<MusicManager>,
    query: Query<Entity, With<MusicEntity>>,
) {
    // If there is no music entity (e.g., was removed externally), ensure we spawn one
    if manager.entity.is_none() {
        // spawn fresh
        let handle = asset_server.load(active_character.menu_music_path());
        let entity = commands
            .spawn((
                AudioPlayer::new(handle),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: Volume::Linear(audio_settings.music_volume),
                    ..default()
                },
                MusicEntity,
            ))
            .id();
        manager.entity = Some(entity);
        manager.current_track = Some(active_character.menu_music_path().to_string());
        return;
    }
    // Only change to the active character's menu track when the active character actually changed.
    if !active_character.is_changed() {
        return;
    }

    let desired = active_character.menu_music_path().to_string();
    if manager.current_track.as_deref() == Some(&desired) {
        return;
    }

    // despawn any existing MusicEntity entities to ensure a clean restart
    for e in &query {
        commands.entity(e).despawn();
    }

    let handle = asset_server.load(active_character.menu_music_path());
    let entity = commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: Volume::Linear(audio_settings.music_volume),
                ..default()
            },
            MusicEntity,
        ))
        .id();

    manager.entity = Some(entity);
    manager.current_track = Some(desired);
}

fn apply_music_volume_change(
    audio_settings: Res<AudioSettings>,
    mut sinks: Query<&mut AudioSink, With<MusicEntity>>,
    mut playbacks_without_sink: Query<&mut PlaybackSettings, (With<MusicEntity>, Without<AudioSink>)>,
) {
    if !audio_settings.is_changed() {
        return;
    }

    let vol = Volume::Linear(audio_settings.music_volume);

    // Runtime audio control must happen through AudioSink.
    for mut sink in &mut sinks {
        sink.set_volume(vol);
    }

    // Keep the initial playback settings in sync for newly spawned music entities
    // that do not yet have an AudioSink attached this frame.
    for mut playback in &mut playbacks_without_sink {
        playback.volume = vol;
    }
}

fn handle_music_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut manager: ResMut<MusicManager>,
    mut request: ResMut<MusicRequest>,
    query: Query<Entity, With<MusicEntity>>,
) {
    let pending = request.0.take();
    let Some(list) = pending else {
        return;
    };

    // replace playlist and start at index 0
    if list.is_empty() {
        return;
    }
    manager.playlist = Some(list.clone());
    manager.playlist_index = 0;

    // Despawn existing music entities so we spawn the first playlist track
    for e in &query {
        commands.entity(e).despawn();
    }

    let desired = manager.playlist.as_ref().unwrap()[0].clone();
    // spawn once (we will advance manually)
    let handle = asset_server.load(desired.clone());
    let entity = commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: Volume::Linear(audio_settings.music_volume),
                ..default()
            },
            MusicEntity,
        ))
        .id();

    manager.entity = Some(entity);
    manager.current_track = Some(desired);
}

fn advance_playlist_if_finished(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut manager: ResMut<MusicManager>,
    sink_query: Query<&AudioSink, With<MusicEntity>>,
    entity_query: Query<Entity, With<MusicEntity>>,
) {
    // Clone the playlist so we don't hold an immutable borrow across
    // a mutable borrow of `manager` below.
    let playlist = match manager.playlist.clone() {
        Some(p) => p,
        None => return,
    };

    // If there is no sink yet (entity not spawned or not attached), nothing to do
    let mut finished = false;
    for sink in &sink_query {
        if sink.empty() {
            finished = true;
            break;
        }
    }

    if !finished {
        return;
    }

    // advance index and wrap
    manager.playlist_index = (manager.playlist_index + 1) % playlist.len();
    let next = playlist[manager.playlist_index].clone();

    // despawn existing music entities
    for e in &entity_query {
        commands.entity(e).despawn();
    }

    let handle = asset_server.load(next.clone());
    let entity = commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: Volume::Linear(audio_settings.music_volume),
                ..default()
            },
            MusicEntity,
        ))
        .id();

    manager.entity = Some(entity);
    manager.current_track = Some(next);
}

