use bevy::prelude::*;

use crate::game::level::types::CachedLevelDefinition;
use crate::{MainCamera, PLAYER_SCREEN_X, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

/// Positions the main camera so the player spawn point appears at
/// x = 30 %, y = 50 % of the virtual viewport, clamped to level bounds.
///
/// The follow_camera system refines this every frame, so this is only
/// the initial placement for the first rendered frame.
pub fn setup_canvas(
    cached: Res<CachedLevelDefinition>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("setup_canvas: no level loaded, skipping camera setup");
        return;
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    // Find the player entity's spawn position.
    let entities = level.entities.as_deref().unwrap_or(&[]);
    let player = entities.iter().find(|e| e.entity_type == "player");
    let player_x = player.map(|e| e.x).unwrap_or(bounds.width / 2.0);
    let player_y = player.map(|e| e.y).unwrap_or(bounds.height / 2.0);

    // Camera center to place the player at (PLAYER_SCREEN_X, 0.5) on screen.
    // With viewport_origin (0.5, 0.5): camera_center_x = player_x + (0.5 - anchor) * vw
    let target_x = player_x + (0.5 - PLAYER_SCREEN_X) * VIRTUAL_WIDTH;
    let target_y = player_y; // 50 % vertically → camera center equals player y

    // Clamp so the viewport never shows outside the level bounds.
    // When the level is smaller than the viewport, anchor to the lower-left corner
    // (camera at half_vw / half_vh places the viewport's left / bottom edge at x=0 / y=0).
    let half_vw = VIRTUAL_WIDTH / 2.0;
    let half_vh = VIRTUAL_HEIGHT / 2.0;

    let cam_x = if bounds.width <= VIRTUAL_WIDTH {
        half_vw
    } else {
        target_x.clamp(half_vw, bounds.width - half_vw)
    };

    let cam_y = if bounds.height <= VIRTUAL_HEIGHT {
        half_vh
    } else {
        target_y.clamp(half_vh, bounds.height - half_vh)
    };

    match camera_query.single_mut() {
        Ok(mut transform) => {
            // Only update X/Y — keep Z at 0 (set by setup_camera) so that
            // menu backgrounds at z=-1 stay within the visible range [-1000, 1000]
            // (near=-1000 with camera at z=0).
            transform.translation.x = cam_x;
            transform.translation.y = cam_y;
            tracing::info!(x = cam_x, y = cam_y, "Camera positioned for level start");
        }
        Err(err) => {
            tracing::warn!("setup_canvas: could not find main camera: {err}");
        }
    }
}

