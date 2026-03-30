use avian2d::prelude::{Collider, RigidBody};
use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::game::components::{self, SpawnedLevelEntity};
use crate::game::level::{
    bottom_left_to_world, clamp_level_position, CachedLevelDefinition,
};
use crate::audio_settings::AudioSettings;
use crate::LevelSelection;

use super::{
    ActiveLevelBounds, CombatSoundEffects, GameViewEntity, LEVEL_BOUNDARY_THICKNESS, LevelQuotes,
    TerrainBackgroundConfig, TerrainBackgroundReady, QuoteCooldown,
};

pub(super) fn setup_game_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    cached_level_definition: Res<CachedLevelDefinition>,
    level_selection: Res<LevelSelection>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mut warnings = Vec::new();
    let mut spawned_count = 0usize;

    let (status_title, status_detail) = match cached_level_definition.level_definition() {
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

            let active_level_bounds = level_bounds
                .map(|level_size| ActiveLevelBounds::from_window_and_level_size(window_size, level_size));

            if let Some(bounds) = active_level_bounds {
                commands.insert_resource(bounds);
            }

            commands.spawn((
                TerrainBackgroundConfig {
                    image: asset_server.load(level_definition.terrain_background_asset_path()),
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
                    AudioPlayer::new(asset_server.load(music_asset_path)),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: bevy::audio::Volume::new(audio_settings.music_volume),
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

                quote_clips.push(asset_server.load(quote_asset_path));
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
                spawn_level_boundaries(&mut commands, bounds);
            }

            for entity_definition in &level_definition.entities {
                let Some(entity_type) = level_definition.entity_types.get(&entity_definition.entity_type)
                else {
                    warnings.push(format!(
                        "{} references unknown entity_type '{}'",
                        entity_definition.id, entity_definition.entity_type
                    ));
                    continue;
                };

                let is_player = entity_type.components.iter().any(|component| component == "player");

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
                    && (level_position.x != entity_definition.x || level_position.y != entity_definition.y)
                {
                    warnings.push(format!(
                        "Clamped player spawn from ({}, {}) to ({}, {}) to fit level bounds",
                        entity_definition.x, entity_definition.y, level_position.x, level_position.y
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
    entity_definition: &crate::game::level::EntityDefinition,
    entity_type: &crate::game::level::EntityTypeDefinition,
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

fn spawn_level_boundaries(commands: &mut Commands, level_bounds: ActiveLevelBounds) {
    let half_thickness = LEVEL_BOUNDARY_THICKNESS * 0.5;
    let vertical_center_y = (level_bounds.bottom + level_bounds.top) * 0.5;
    let horizontal_center_x = (level_bounds.left + level_bounds.right) * 0.5;
    let vertical_wall_height =
        (level_bounds.top - level_bounds.bottom) + LEVEL_BOUNDARY_THICKNESS * 2.0;
    let horizontal_wall_width =
        (level_bounds.right - level_bounds.left) + LEVEL_BOUNDARY_THICKNESS * 2.0;

    let walls = [
        (
            "Left",
            Vec3::new(level_bounds.left - half_thickness, vertical_center_y, 50.0),
            Vec2::new(LEVEL_BOUNDARY_THICKNESS, vertical_wall_height),
        ),
        (
            "Right",
            Vec3::new(level_bounds.right + half_thickness, vertical_center_y, 50.0),
            Vec2::new(LEVEL_BOUNDARY_THICKNESS, vertical_wall_height),
        ),
        (
            "Bottom",
            Vec3::new(horizontal_center_x, level_bounds.bottom - half_thickness, 50.0),
            Vec2::new(horizontal_wall_width, LEVEL_BOUNDARY_THICKNESS),
        ),
        (
            "Top",
            Vec3::new(horizontal_center_x, level_bounds.top + half_thickness, 50.0),
            Vec2::new(horizontal_wall_width, LEVEL_BOUNDARY_THICKNESS),
        ),
    ];

    for (name, translation, size) in walls {
        commands.spawn((
            Name::new(format!("LevelBoundary:{name}")),
            Transform::from_translation(translation),
            Collider::rectangle(size.x, size.y),
            RigidBody::Static,
            SpawnedLevelEntity,
        ));
    }
}

pub(super) fn spawn_terrain_background_tiles(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    active_level_bounds: Option<Res<ActiveLevelBounds>>,
    configs: Query<(Entity, &TerrainBackgroundConfig), Without<TerrainBackgroundReady>>,
) {
    let window = windows.single();

    for (entity, config) in &configs {
        let Some(image) = images.get(&config.image) else {
            continue;
        };

        let image_width = image.texture_descriptor.size.width as f32;
        let image_height = image.texture_descriptor.size.height as f32;

        if image_width <= 0.0 || image_height <= 0.0 {
            continue;
        }

        let tile_height = window.height();
        let tile_width = (image_width / image_height) * tile_height;
        let (start_x, span_width, start_y) = match active_level_bounds.as_deref().copied() {
            Some(bounds) => (bounds.left, bounds.right - bounds.left, bounds.bottom),
            None => (-(window.width() * 0.5), window.width(), -(window.height() * 0.5)),
        };
        let tile_count = ((span_width / tile_width).ceil() as usize).saturating_add(1);

        for index in 0..tile_count {
            let x = start_x + (index as f32 * tile_width);
            let y = start_y;

            commands.spawn((
                Sprite {
                    image: config.image.clone(),
                    custom_size: Some(Vec2::new(tile_width, tile_height)),
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                Transform::from_xyz(x, y, -100.0),
                super::parallax::BackgroundParallax,
                GameViewEntity,
            ));
        }

        commands.entity(entity).insert(TerrainBackgroundReady);
    }
}

fn spawn_overlay(
    commands: &mut Commands,
    status_title: String,
    status_detail: String,
    warnings: &[String],
) {
    let warning_text = if warnings.is_empty() {
        "No component warnings".to_string()
    } else {
        format!("Warnings: {}", warnings.join(" | "))
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Px(8.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            Visibility::Hidden,
            super::DebugOverlayRoot,
            GameViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game View"),
                TextFont {
                    font_size: 38.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(status_title),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.7, 1.0)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(status_detail),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(warning_text),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.35)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new("Press O to toggle overlay | Press L to toggle hitboxes | Press Esc to return"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                GameViewEntity,
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::game::level::{EntityDefinition, EntityTypeDefinition};

    fn entity_type_with_components(components: &[&str]) -> EntityTypeDefinition {
        EntityTypeDefinition {
            components: components.iter().map(|component| component.to_string()).collect(),
            disposition: None,
            states: HashMap::new(),
            width: 16.0,
            height: 16.0,
            health: None,
            damage: None,
            attack_range: None,
        }
    }


    #[test]
    fn uses_explicit_entity_z_index_when_present() {
        let entity_definition = EntityDefinition {
            id: "crate1".to_string(),
            entity_type: "crate".to_string(),
            x: 0.0,
            y: 0.0,
            z_index: Some(7.0),
        };
        let entity_type = entity_type_with_components(&["doodad"]);

        let z = resolve_entity_z_index(&entity_definition, &entity_type, false);

        assert_eq!(z, 7.0);
    }

    #[test]
    fn falls_back_to_component_based_z_index_when_entity_value_is_missing() {
        let player_definition = EntityDefinition {
            id: "player".to_string(),
            entity_type: "bob".to_string(),
            x: 0.0,
            y: 0.0,
            z_index: None,
        };
        let npc_definition = EntityDefinition {
            id: "roach".to_string(),
            entity_type: "cockroach".to_string(),
            x: 0.0,
            y: 0.0,
            z_index: None,
        };
        let doodad_definition = EntityDefinition {
            id: "crate1".to_string(),
            entity_type: "crate".to_string(),
            x: 0.0,
            y: 0.0,
            z_index: None,
        };

        let player_z = resolve_entity_z_index(&player_definition, &entity_type_with_components(&["player"]), true);
        let npc_z = resolve_entity_z_index(&npc_definition, &entity_type_with_components(&["npc"]), false);
        let doodad_z = resolve_entity_z_index(&doodad_definition, &entity_type_with_components(&["doodad"]), false);

        assert_eq!(player_z, 20.0);
        assert_eq!(npc_z, 10.0);
        assert_eq!(doodad_z, 0.0);
    }
}




