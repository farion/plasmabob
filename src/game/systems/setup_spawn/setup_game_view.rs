use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::components::{self};
use crate::game::systems::presentation::types::LevelTimer;
use crate::game::systems::setup_spawn::spawn_level_boundaries;
use crate::game::systems::setup_spawn::spawn_overlay::spawn_overlay;
use crate::game::systems::systems_api::{
    ActiveLevelBounds, CombatSoundEffects, GameViewEntity, LevelQuotes, QuoteCooldown,
    TerrainBackgroundConfig,
};
use crate::helper::audio_settings::AudioSettings;
use crate::level::{CachedLevelDefinition, bottom_left_to_world, clamp_level_position};
use crate::{LevelSelection, LevelStats};

pub(crate) fn setup_game_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    cached_level_definition: Res<CachedLevelDefinition>,
    level_selection: Res<LevelSelection>,
    mut stats: ResMut<LevelStats>,
    mut level_timer: ResMut<LevelTimer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    *stats = LevelStats::default();
    level_timer.0.reset();

    let Ok(window) = windows.single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());
    let mut warnings = Vec::new();
    let mut spawned_count = 0usize;

    let (status_title, status_detail) = match cached_level_definition.level_definition().cloned() {
        Ok(level_definition) => {
            let mut quote_clips = Vec::new();
            let level_bounds = match level_definition.bounds_size() {
                Some(bounds) if bounds.x > 0.0 && bounds.y > 0.0 => Some(bounds),
                Some(bounds) => {
                    warnings.push(format!(
                        "Ignoring invalid level bounds {}x{}; both values must be > 0",
                        bounds.x, bounds.y
                    ));
                    None
                }
                None => None,
            };

            let active_level_bounds = level_bounds.map(|level_size| {
                ActiveLevelBounds::from_window_and_level_size(window_size, level_size)
            });

            if let Some(bounds) = active_level_bounds {
                commands.insert_resource(bounds);
            }

            commands.spawn((
                TerrainBackgroundConfig {
                    image: asset_server
                        .load(level_definition.terrain_background_asset_path().to_string()),
                },
                GameViewEntity,
            ));

            let music_asset_path = level_definition.music_asset_path();
            if !music_asset_path.ends_with(".ogg") {
                warnings.push(format!(
                    "Level music '{}' is invalid: only .ogg is supported",
                    music_asset_path
                ));
            } else {
                commands.spawn((
                    AudioPlayer::new(asset_server.load(music_asset_path.to_string())),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: bevy::audio::Volume::Linear(audio_settings.music_volume),
                        ..default()
                    },
                    GameViewEntity,
                ));
            }

            for quote_asset_path in level_definition.quote_asset_paths() {
                if !quote_asset_path.ends_with(".ogg") {
                    warnings.push(format!(
                        "Quote '{}' is invalid: only .ogg is supported",
                        quote_asset_path
                    ));
                    continue;
                }

                quote_clips.push(asset_server.load(quote_asset_path.to_string()));
            }

            commands.insert_resource(LevelQuotes { clips: quote_clips });
            commands.insert_resource(CombatSoundEffects {
                plasma_shot: asset_server.load("audio/plasma-shot.ogg"),
                plasma_hit: asset_server.load("audio/plasma-hit.ogg"),
                cockroach_die: asset_server.load("audio/cockroach-die.ogg"),
            });
            // Ensure the quote cooldown is reset when a level is (re)loaded so
            // death quotes can play immediately according to the default timer.
            commands.insert_resource(QuoteCooldown::default());

            if let Some(bounds) = active_level_bounds {
                spawn_level_boundaries::spawn_level_boundaries(&mut commands, bounds);
            }

            for entity_definition in &level_definition.entities {
                let Some(entity_type) = level_definition
                    .entity_types
                    .get(&entity_definition.entity_type)
                else {
                    warnings.push(format!(
                        "{} references unknown entity_type '{}',",
                        entity_definition.id, entity_definition.entity_type
                    ));
                    continue;
                };

                let is_player = entity_type
                    .components
                    .iter()
                    .any(|component| component == "player");

                // Resolve z-index: per-entity value or a component-based fallback.
                let z = resolve_entity_z_index(entity_definition, entity_type, is_player);

                let level_position = if is_player {
                    level_bounds
                        .map(|level_size| {
                            clamp_level_position(
                                entity_definition.x,
                                entity_definition.y,
                                entity_type.size(),
                                level_size,
                            )
                        })
                        .unwrap_or(Vec2::new(entity_definition.x, entity_definition.y))
                } else {
                    Vec2::new(entity_definition.x, entity_definition.y)
                };

                if is_player
                    && (level_position.x != entity_definition.x
                        || level_position.y != entity_definition.y)
                {
                    warnings.push(format!(
                        "Clamped player spawn from ({}, {}) to ({}, {}) to fit level bounds",
                        entity_definition.x,
                        entity_definition.y,
                        level_position.x,
                        level_position.y
                    ));
                }

                let world_position = bottom_left_to_world(
                    window_size,
                    level_position.x,
                    level_position.y,
                    entity_type.size(),
                    z,
                );

                warnings.extend(components::spawn_entity(
                    &mut commands,
                    &asset_server,
                    entity_definition,
                    entity_type,
                    world_position,
                ));
                spawned_count += 1;
            }

            (
                format!("Loaded {spawned_count} entities from {}", level_selection.asset_path()),
                match level_bounds {
                    Some(level_size) => format!(
                        "Level origin is bottom-left (0,0). Boundaries: {} x {}. Camera keeps Bob at 40% screen width when possible.",
                        level_size.x, level_size.y
                    ),
                    None => {
                        "Level origin is bottom-left (0,0). No boundaries defined; camera follows Bob freely."
                            .to_string()
                    }
                },
            )
        }
        Err(error) => (
            format!("Could not load {}", level_selection.asset_path()),
            error.to_string(),
        ),
    };

    spawn_overlay(&mut commands, status_title, status_detail, &warnings);
}

fn resolve_entity_z_index(
    entity_definition: &crate::level::EntityDefinition,
    entity_type: &crate::level::EntityTypeDefinition,
    is_player: bool,
) -> f32 {
    entity_definition.z_index.unwrap_or_else(|| {
        if is_player {
            20.0
        } else if entity_type.components.iter().any(|c| c == "npc") {
            10.0
        } else {
            0.0
        }
    })
}
