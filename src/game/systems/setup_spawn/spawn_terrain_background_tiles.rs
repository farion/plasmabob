use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::game::systems::presentation::types::BackgroundParallax;
use crate::game::systems::systems_api::{
    ActiveLevelBounds, GameViewEntity, TerrainBackgroundConfig, TerrainBackgroundReady,
};

pub(crate) fn spawn_terrain_background_tiles(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    active_level_bounds: Option<Res<ActiveLevelBounds>>,
    configs: Query<(Entity, &TerrainBackgroundConfig), Without<TerrainBackgroundReady>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

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
            None => (
                -(window.width() * 0.5),
                window.width(),
                -(window.height() * 0.5),
            ),
        };
        let tile_count = ((span_width / tile_width).ceil() as usize).saturating_add(1);

        for index in 0..tile_count {
            let x = start_x + (index as f32 * tile_width);
            let y = start_y;

            commands.spawn((
                Sprite {
                    image: config.image.clone(),
                    custom_size: Some(Vec2::new(tile_width, tile_height)),
                    ..default()
                },
                Anchor::BOTTOM_LEFT,
                Transform::from_xyz(x, y, -100.0),
                BackgroundParallax,
                GameViewEntity,
            ));
        }

        commands.entity(entity).insert(TerrainBackgroundReady);
    }
}
