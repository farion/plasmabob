use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::MainCamera;

use super::{ActiveLevelBounds, GameViewEntity};

pub(super) fn cleanup_game_view(
    mut commands: Commands,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    ui_entities: Query<Entity, With<GameViewEntity>>,
    level_entities: Query<Entity, With<SpawnedLevelEntity>>,
) {
    commands.remove_resource::<ActiveLevelBounds>();

    if let Ok(mut camera_transform) = cameras.get_single_mut() {
        camera_transform.translation.x = 0.0;
        camera_transform.translation.y = 0.0;
    }

    for entity in &ui_entities {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &level_entities {
        commands.entity(entity).despawn_recursive();
    }
}



