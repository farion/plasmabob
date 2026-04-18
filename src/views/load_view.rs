use bevy::asset::LoadState;
use bevy::audio::AudioSource;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::app_model::AppState;
use crate::game::level::types::CachedLevelDefinition;
use crate::game::setup::entity_type_assets::{EntityTypeAsset, EntityTypeAssets, StateAssets};
use crate::helper::active_character::ActiveCharacter;
use crate::helper::asset_io::{load_character_asset, resolve_character_asset_path};
use crate::helper::music::MusicRequest;
use crate::LevelSelection;

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub(crate) struct LoadViewPlugin;

impl Plugin for LoadViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadView), setup_load_view)
            .add_systems(
                Update,
                tick_load_view.run_if(in_state(AppState::LoadView)),
            )
            .add_systems(OnExit(AppState::LoadView), cleanup_load_view);
    }
}

// ─── Resources ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadPhase {
    /// Waiting for all sprite handles to become available.
    Sprites,
    /// Waiting for all audio handles to become available.
    Sounds,
    /// All assets ready — build the cache and transition to GameView.
    Done,
}

struct SoundEntry {
    path: String,
    handle: Handle<AudioSource>,
    duration_secs: f32,
    failed_logged: bool,
}

#[derive(Resource)]
struct LoadProgress {
    phase: LoadPhase,
    // Sprite phase
    sprite_paths: Vec<String>,
    sprite_handles: Vec<Handle<Image>>,
    // Sound phase (populated lazily when sprites are done)
    sound_paths: Vec<String>,
    sound_entries: Vec<SoundEntry>,
    sounds_issued: bool,
}

// ─── Marker ───────────────────────────────────────────────────────────────────

#[derive(Component)]
struct LoadViewEntity;

#[derive(Component)]
struct LoadProgressText;

// ─── Setup ────────────────────────────────────────────────────────────────────

fn setup_load_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_character: Res<ActiveCharacter>,
    level_selection: Res<LevelSelection>,
    existing_cached: Option<Res<CachedLevelDefinition>>,
    mut music_request: ResMut<MusicRequest>,
) {
    // Load level JSON synchronously if not already cached.
    let cached: CachedLevelDefinition = if existing_cached.is_some() {
        // Already present — we'll read it from the resource in tick_load_view.
        // Still need to collect sprite paths, so clone via match below.
        match crate::game::level::loader::load_level_from_asset(
            &asset_server,
            level_selection.asset_path(),
            *active_character,
        ) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = ?e, "LoadView: failed to load level JSON");
                CachedLevelDefinition::default()
            }
        }
    } else {
        match crate::game::level::loader::load_level_from_asset(
            &asset_server,
            level_selection.asset_path(),
            *active_character,
        ) {
            Ok(c) => {
                // Set up music playlist.
                if let Some(music) = c.level.as_ref().and_then(|l| l.music.clone()) {
                    if !music.is_empty() {
                        music_request.0 = Some(music);
                    }
                }
                c
            }
            Err(e) => {
                tracing::error!(error = ?e, "LoadView: failed to load level JSON");
                CachedLevelDefinition::default()
            }
        }
    };

    // Insert the freshly loaded definition (overwrites any stale previous one).
    commands.insert_resource(cached.clone());

    // Collect unique sprite paths.
    let sprite_paths = collect_sprite_paths(&cached);
    let sprite_handles: Vec<Handle<Image>> = sprite_paths
        .iter()
        .map(|p| load_character_asset::<Image>(&asset_server, p, *active_character))
        .collect();
    let sound_paths = collect_sound_paths(&cached);

    tracing::info!(
        sprites = sprite_paths.len(),
        sounds = sound_paths.len(),
        "LoadView: starting asset loading"
    );

    // Spawn loading UI.
    let _text_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            LoadViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading sprites 0%"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::WHITE),
                LoadProgressText,
                LoadViewEntity,
            ));
        })
        .id();

    commands.insert_resource(LoadProgress {
        phase: LoadPhase::Sprites,
        sprite_paths,
        sprite_handles,
        sound_paths,
        sound_entries: vec![],
        sounds_issued: false,
    });
}

// ─── Update ───────────────────────────────────────────────────────────────────

fn tick_load_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_character: Res<ActiveCharacter>,
    images: Res<Assets<Image>>,
    audio_assets: Res<Assets<AudioSource>>,
    cached: Option<Res<CachedLevelDefinition>>,
    mut progress: ResMut<LoadProgress>,
    mut text_query: Query<&mut Text, With<LoadProgressText>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match progress.phase {
        // ── Phase A: wait for sprite handles ─────────────────────────────
        LoadPhase::Sprites => {
            let loaded = progress
                .sprite_handles
                .iter()
                .filter(|h| images.contains(h.id()))
                .count();
            let total = progress.sprite_handles.len().max(1);
            let pct = (loaded * 100 / total) as u8;

            update_text(&mut text_query, &format!("Loading sprites {}%", pct));

            if loaded >= progress.sprite_handles.len() {
                tracing::info!("LoadView: all sprites loaded, starting sound phase");
                progress.phase = LoadPhase::Sounds;
            }
        }

        // ── Phase B: issue sound loads, compute durations, then wait ─────
        LoadPhase::Sounds => {
            // Issue loads on the first Sounds frame.
            if !progress.sounds_issued {
                // Collect paths first to avoid borrow conflict with sound_entries.
                let paths: Vec<String> = progress.sound_paths.clone();
                for path in &paths {
                    let resolved = resolve_character_asset_path(&asset_server, path, *active_character)
                        .unwrap_or_else(|_| path.clone());

                    if !find_assets_dir().join(&resolved).exists() {
                        tracing::warn!(path = %path, "LoadView: sound not found - ignoring");
                        continue;
                    }

                    let handle: Handle<AudioSource> =
                        load_character_asset::<AudioSource>(&asset_server, path, *active_character);
                    let duration_secs = probe_audio_duration(&resolved).unwrap_or_else(|| {
                        tracing::warn!(path = %path, "LoadView: could not probe sound duration, using 0");
                        0.0
                    });
                    progress.sound_entries.push(SoundEntry {
                        path: path.clone(),
                        handle,
                        duration_secs,
                        failed_logged: false,
                    });
                }
                progress.sounds_issued = true;
            }

            let mut loaded_or_ignored = 0usize;
            for entry in &mut progress.sound_entries {
                if audio_assets.contains(entry.handle.id()) {
                    loaded_or_ignored += 1;
                    continue;
                }

                if let Some(LoadState::Failed(err)) = asset_server.get_load_state(entry.handle.id()) {
                    loaded_or_ignored += 1;
                    if !entry.failed_logged {
                        tracing::warn!(path = %entry.path, error = %err, "LoadView: sound failed to load - ignoring");
                        entry.failed_logged = true;
                    }
                }
            }

            let total = progress.sound_entries.len().max(1);
            let pct = (loaded_or_ignored * 100 / total) as u8;

            update_text(&mut text_query, &format!("Loading sounds {}%", pct));

            if loaded_or_ignored >= progress.sound_entries.len() {
                tracing::info!("LoadView: all sounds processed, building cache");
                progress.phase = LoadPhase::Done;
            }
        }

        // ── Phase C: build EntityTypeAssets and enter GameView ───────────
        LoadPhase::Done => {
            let Some(cached_res) = cached else {
                return;
            };

            // Build path → handle maps.
            let sprite_map: HashMap<String, Handle<Image>> = progress
                .sprite_paths
                .iter()
                .zip(progress.sprite_handles.iter())
                .map(|(p, h)| (p.clone(), h.clone()))
                .collect();

            let sound_map: HashMap<String, (Handle<AudioSource>, f32)> = progress
                .sound_entries
                .iter()
                .filter(|e| audio_assets.contains(e.handle.id()))
                .map(|e| (e.path.clone(), (e.handle.clone(), e.duration_secs)))
                .collect();

            // Warn about sprites that failed to load.
            for (path, handle) in &sprite_map {
                if !images.contains(handle.id()) {
                    tracing::warn!(path = %path, "LoadView: sprite not fully loaded — using red fallback");
                }
            }

            let entity_type_assets =
                build_entity_type_assets(&cached_res, &sprite_map, &sound_map);
            commands.insert_resource(entity_type_assets);

            next_state.set(AppState::GameView);
        }
    }
}

// ─── Cleanup ──────────────────────────────────────────────────────────────────

fn cleanup_load_view(
    mut commands: Commands,
    entities: Query<Entity, With<LoadViewEntity>>,
) {
    for e in &entities {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<LoadProgress>();
}

// ─── Asset path collection ────────────────────────────────────────────────────

fn collect_sprite_paths(cached: &CachedLevelDefinition) -> Vec<String> {
    let mut paths: HashSet<String> = HashSet::new();
    for (_, et) in &cached.entity_types {
        if let Some(sm) = et.state_machine_config() {
            for (_, sc) in &sm.states {
                for p in &sc.animation {
                    paths.insert(p.clone());
                }
            }
        }
    }
    paths.into_iter().collect()
}

fn collect_sound_paths(cached: &CachedLevelDefinition) -> Vec<String> {
    let mut paths: HashSet<String> = HashSet::new();
    for (_, et) in &cached.entity_types {
        if let Some(sm) = et.state_machine_config() {
            for (_, sc) in &sm.states {
                if let Some(p) = &sc.sound_start { paths.insert(p.clone()); }
                if let Some(p) = &sc.sound_loop  { paths.insert(p.clone()); }
                if let Some(p) = &sc.sound_end   { paths.insert(p.clone()); }
            }
        }
    }
    paths.into_iter().collect()
}

// ─── EntityTypeAssets builder ─────────────────────────────────────────────────

fn build_entity_type_assets(
    cached: &CachedLevelDefinition,
    sprite_map: &HashMap<String, Handle<Image>>,
    sound_map: &HashMap<String, (Handle<AudioSource>, f32)>,
) -> EntityTypeAssets {
    let mut result = EntityTypeAssets::default();

    for (et_name, et_def) in &cached.entity_types {
        let Some(sm_cfg) = et_def.state_machine_config() else { continue };

        let sprite_w = et_def.width.unwrap_or(128.0);
        let sprite_h = et_def.height.unwrap_or(128.0);
        let fallback = sm_cfg.initial_state.to_ascii_lowercase();

        let mut states: HashMap<String, StateAssets> = HashMap::new();

        for (state_name, sc) in &sm_cfg.states {
            let sname = state_name.to_ascii_lowercase();

            let frames: Vec<Handle<Image>> = sc.animation.iter()
                .filter_map(|p| sprite_map.get(p).cloned())
                .collect();

            let sound_start = sc.sound_start.as_ref()
                .and_then(|p| sound_map.get(p))
                .map(|(h, d)| (h.clone(), *d));

            let sound_loop = sc.sound_loop.as_ref()
                .and_then(|p| sound_map.get(p))
                .map(|(h, _)| h.clone());

            let sound_end = sc.sound_end.as_ref()
                .and_then(|p| sound_map.get(p))
                .map(|(h, _)| h.clone());

            states.insert(sname, StateAssets {
                frames,
                animation_frame_ms: sc.animation_frame_ms,
                lock_ms: sc.lock_ms,
                collider_box: sc.collider_box.clone(),
                sound_start,
                sound_loop,
                sound_end,
            });
        }

        // Fill empty-frame states from the fallback state (AGENTS.md spec).
        let fallback_frames: Vec<Handle<Image>> = states
            .get(&fallback)
            .map(|s| s.frames.clone())
            .unwrap_or_default();

        for sa in states.values_mut() {
            if sa.frames.is_empty() && !fallback_frames.is_empty() {
                sa.frames = fallback_frames.clone();
            }
        }

        result.map.insert(et_name.clone(), EntityTypeAsset {
            states,
            fallback_state: fallback,
            sprite_width: sprite_w,
            sprite_height: sprite_h,
        });
    }

    result
}

// ─── Symphonia duration probe ─────────────────────────────────────────────────

/// Probe the duration of an audio file by path (relative to the `assets/` directory).
/// Returns `None` if the file cannot be found or parsed.
fn probe_audio_duration(asset_path: &str) -> Option<f32> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let base = find_assets_dir();
    let full_path = base.join(asset_path);

    let file = std::fs::File::open(&full_path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = full_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .ok()?;

    let format = probed.format;
    let track = format.default_track()?;
    let sample_rate = track.codec_params.sample_rate? as f32;
    let n_frames = track.codec_params.n_frames? as f32;
    Some(n_frames / sample_rate)
}

fn find_assets_dir() -> std::path::PathBuf {
    // Development: CARGO_MANIFEST_DIR/assets
    let dev = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if dev.exists() {
        return dev;
    }
    // Production: executable directory / assets
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("assets");
            if p.exists() {
                return p;
            }
        }
    }
    std::path::PathBuf::from("assets")
}

// ─── UI helper ────────────────────────────────────────────────────────────────

fn update_text(query: &mut Query<&mut Text, With<LoadProgressText>>, msg: &str) {
    for mut text in query.iter_mut() {
        text.0 = msg.to_string();
    }
}


