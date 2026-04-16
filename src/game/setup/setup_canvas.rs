use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use crate::game::level::types::CachedLevelDefinition;
use crate::game::setup::follow_camera::camera_center_for_player;
use crate::MainCamera;

/// Positions the main camera so the player spawn point appears at
/// x = 30 %, y = 50 % of the virtual viewport, clamped to level bounds.
///
/// The follow_camera system refines this every frame, so this is only
/// the initial placement for the first rendered frame.
pub fn setup_canvas(
    cached: Res<CachedLevelDefinition>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Projection), With<MainCamera>>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("setup_canvas: no level loaded, skipping camera setup");
        return;
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    // Find the player entity's spawn position.
    let entities = level.entities.as_deref().unwrap_or(&[]);
    let player = entities.iter().find(|e| e.entity_type.key == "player");
    let player_x = player.map(|e| e.x).unwrap_or(bounds.width / 2.0);
    let player_y = player.map(|e| e.y).unwrap_or(bounds.height / 2.0);
    let window = windows.single().ok();

    match camera_query.single_mut() {
        Ok((mut transform, projection)) => {
            let cam = camera_center_for_player(player_x, player_y, projection, window, Some(&bounds));

            // Only update X/Y — keep Z at 0 (set by setup_camera) so that
            // menu backgrounds at z=-1 stay within the visible range [-1000, 1000]
            // (near=-1000 with camera at z=0).
            transform.translation.x = cam.x;
            transform.translation.y = cam.y;
            tracing::info!(x = cam.x, y = cam.y, "Camera positioned for level start");
        }
        Err(err) => {
            tracing::warn!("setup_canvas: could not find main camera: {err}");
        }
    }
}

