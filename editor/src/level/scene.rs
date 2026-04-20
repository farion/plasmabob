use crate::core::LevelFile;
use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use crate::level::helper::{
    entity_render_center, resolve_character_asset_path, z_overlay_color_for_value,
};
use crate::level::run::{
    ActiveCharacter, BackgroundTilesReady, EditorCamera, PendingBackgroundTiles,
    RenderedLevelEntity, RenderedZOverlay, SceneEntity,
};

use crate::core::EntityTypeDefinition;
use crate::level::state::EntityTypeViewState;

pub fn spawn_background(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: &LevelFile,
    level_size: Vec2,
    active: ActiveCharacter,
) {
    // moved from editor.rs
    let background_candidate = level
        .background
        .as_deref()
        .filter(|path| !path.is_empty())
        .or_else(|| {
            level
                .terrain
                .as_ref()
                .and_then(|terrain| terrain.background.as_deref())
                .filter(|path| !path.is_empty())
        });

    let background_path = if let Some(bp) = background_candidate {
        match resolve_character_asset_path(asset_server, bp, active) {
            Ok(p) => p,
            Err(_) => crate::core::normalize_asset_reference(bp),
        }
    } else {
        String::new()
    };

    let image = asset_server.load(background_path);

    commands.spawn((SceneEntity, PendingBackgroundTiles { image, level_size }));
}

pub fn spawn_background_tiles_when_ready(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    pending_backgrounds: Query<(Entity, &PendingBackgroundTiles), Without<BackgroundTilesReady>>,
) {
    for (entity, pending) in &pending_backgrounds {
        let Some(image) = images.get(&pending.image) else {
            continue;
        };

        let image_width = image.texture_descriptor.size.width as f32;
        let image_height = image.texture_descriptor.size.height as f32;
        if image_width <= 0.0 || image_height <= 0.0 {
            continue;
        }

        let tile_height = pending.level_size.y.max(1.0);
        let tile_width = (image_width / image_height) * tile_height;
        let tile_count = ((pending.level_size.x / tile_width).ceil() as usize).saturating_add(1);

        for index in 0..tile_count {
            let mut sprite = Sprite::from_image(pending.image.clone());
            sprite.custom_size = Some(Vec2::new(tile_width, tile_height));
            let x = index as f32 * tile_width + tile_width * 0.5;
            let y = tile_height * 0.5;

            commands.spawn((SceneEntity, sprite, Transform::from_xyz(x, y, -10.0)));
        }

        commands.entity(entity).insert(BackgroundTilesReady);
    }
}

pub fn spawn_level_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
    spawn_z_overlays: bool,
    active: ActiveCharacter,
) {
    for (index, entity) in level.entities.iter().enumerate() {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            warn!("entity type '{}' not found", entity.entity_type);
            continue;
        };

        let size = entity_type.size();
        let z = entity.z_index.unwrap_or(100.0);
        let render_position = entity_render_center(Vec2::new(entity.x, entity.y), size);
        let transform = Transform::from_xyz(render_position.x, render_position.y, z);

        if let Some(texture_path) = entity_type.default_texture_asset_path() {
            let normalized = crate::core::normalize_asset_reference(&texture_path);
            let resolved = match resolve_character_asset_path(asset_server, &normalized, active) {
                Ok(p) => p,
                Err(_) => normalized.clone(),
            };
            let mut sprite = Sprite::from_image(asset_server.load(resolved));
            sprite.custom_size = Some(size);
            commands.spawn((
                SceneEntity,
                RenderedLevelEntity { index },
                sprite,
                transform,
            ));
        } else {
            let sprite = Sprite::from_color(Color::srgba(0.4, 0.6, 1.0, 0.9), size);
            commands.spawn((
                SceneEntity,
                RenderedLevelEntity { index },
                sprite,
                transform,
            ));
        }

        if spawn_z_overlays {
            let overlay_sprite = Sprite::from_color(z_overlay_color_for_value(z), size);
            commands.spawn((
                SceneEntity,
                RenderedZOverlay { index },
                overlay_sprite,
                Transform::from_xyz(render_position.x, render_position.y, z + 0.01),
            ));
        }
    }
}

pub fn rebuild_scene_if_needed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_dirty: ResMut<crate::level::state::SceneDirty>,
    mut camera_fit_requested: ResMut<crate::level::state::CameraFitRequested>,
    overlay_mode: Res<crate::level::state::ZOverlayMode>,
    document: Res<crate::level::state::EditorDocument>,
    scene_entities: Query<Entity, With<SceneEntity>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    active_character: Res<ActiveCharacter>,
) {
    if !scene_dirty.0 {
        return;
    }

    for entity in &scene_entities {
        commands.entity(entity).despawn();
    }

    let document_level_size = level_size(&document.level, &document.entity_types);
    let active = *active_character;
    spawn_background(
        &mut commands,
        &asset_server,
        &document.level,
        document_level_size,
        active,
    );
    spawn_level_entities(
        &mut commands,
        &asset_server,
        &document.level,
        &document.entity_types,
        overlay_mode.enabled,
        active,
    );
    if camera_fit_requested.0 {
        fit_camera_to_level(
            &document.level,
            &document.entity_types,
            &window_query,
            &mut camera_query,
        );
        camera_fit_requested.0 = false;
    }

    scene_dirty.0 = false;
}

pub fn fit_camera_to_level(
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &mut Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let level_size = level_size(level, entity_types).max(Vec2::new(100.0, 100.0));
    transform.translation.x = level_size.x * 0.5;
    transform.translation.y = level_size.y * 0.5;

    let scale_x = level_size.x / window.width().max(1.0);
    let scale_y = level_size.y / window.height().max(1.0);
    if let Projection::Orthographic(orthographic) = projection.as_mut() {
        orthographic.scale = scale_x.max(scale_y).max(0.2) * 1.05;
    }
}

pub fn level_size(level: &LevelFile, entity_types: &HashMap<String, EntityTypeDefinition>) -> Vec2 {
    if let Some(bounds) = &level.bounds {
        return bounds.size();
    }

    let mut max_corner = Vec2::ZERO;
    for entity in &level.entities {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            continue;
        };
        let size = entity_type.size();
        max_corner.x = max_corner.x.max(entity.x + size.x);
        max_corner.y = max_corner.y.max(entity.y + size.y);
    }

    max_corner.max(Vec2::new(100.0, 100.0))
}
