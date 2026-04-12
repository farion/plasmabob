use bevy::prelude::*;

use crate::game::level::types::{CachedLevelDefinition, LevelBounds};
use crate::MainCamera;

/// Positions the main camera at the player spawn location so the level start
/// is visible when the GameView is entered.
///
/// The camera is placed at the player entity's X position and at half the
/// level height vertically, giving a balanced view of the level.
pub fn setup_canvas(
    cached: Res<CachedLevelDefinition>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("setup_canvas: no level loaded, skipping camera setup");
        return;
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    // Locate the player entity to start the camera on its X position.
    let player_x = level
        .entities
        .as_deref()
        .and_then(|ents| ents.iter().find(|e| e.entity_type == "player"))
        .map(|e| e.x)
        .unwrap_or(bounds.width / 2.0);

    // Clamp so the camera never shows outside the level.
    let cam_x = player_x.clamp(0.0, bounds.width);
    let cam_y = bounds.height / 2.0;

    match camera_query.single_mut() {
        Ok(mut transform) => {
            transform.translation = Vec3::new(cam_x, cam_y, 999.0);
            tracing::info!(x = cam_x, y = cam_y, "Camera positioned for level start");
        }
        Err(err) => {
            tracing::warn!("setup_canvas: could not find main camera: {err}");
        }
    }
}

