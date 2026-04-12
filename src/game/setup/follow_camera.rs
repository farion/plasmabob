use bevy::prelude::*;

use crate::game::gameplay::components::PlayerTag;
use crate::game::level::types::CachedLevelDefinition;
use crate::{MainCamera, PLAYER_SCREEN_X, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

/// Every frame: move the camera so the player appears at
/// (PLAYER_SCREEN_X, 0.5) = (30 %, 50 %) of the viewport,
/// unless doing so would show outside the level bounds.
///
/// The actual viewport size in world units is read from the camera's
/// `OrthographicProjection::area`, so the system is resolution-independent.
pub fn follow_camera(
    players: Query<&Transform, (With<PlayerTag>, Without<MainCamera>)>,
    mut cameras: Query<
        (&mut Transform, &Projection),
        (With<MainCamera>, Without<PlayerTag>),
    >,
    cached: Option<Res<CachedLevelDefinition>>,
) {
    let Ok(player_tf) = players.single() else {
        return;
    };
    let Ok((mut cam_tf, projection)) = cameras.single_mut() else {
        return;
    };

    // Derive viewport dimensions from the orthographic projection area.
    // Falls back to the virtual constants if the projection is not orthographic.
    let (vw, vh) = match projection {
        Projection::Orthographic(ortho) => {
            let w = ortho.area.width();
            let h = ortho.area.height();
            if w > 0.0 && h > 0.0 {
                (w, h)
            } else {
                (VIRTUAL_WIDTH, VIRTUAL_HEIGHT)
            }
        }
        _ => (VIRTUAL_WIDTH, VIRTUAL_HEIGHT),
    };

    let px = player_tf.translation.x;
    let py = player_tf.translation.y;

    // Ideal camera centre so the player sits at (PLAYER_SCREEN_X, 0.5).
    // With viewport_origin (0.5, 0.5):  camera_x = player_x + (0.5 - anchor) * vw
    let target_x = px + (0.5 - PLAYER_SCREEN_X) * vw;
    let target_y = py; // 50 % vertically → camera centre = player y

    // Clamp so the viewport never exceeds the level bounds.
    let level_bounds = cached
        .as_deref()
        .and_then(|c| c.level.as_ref())
        .and_then(|l| l.bounds.clone());

    let half_vw = vw / 2.0;
    let half_vh = vh / 2.0;

    let cam_x = if let Some(ref b) = level_bounds {
        if b.width <= vw {
            // Level narrower than viewport → anchor to left edge (x = 0).
            half_vw
        } else {
            target_x.clamp(half_vw, b.width - half_vw)
        }
    } else {
        target_x
    };

    let cam_y = if let Some(ref b) = level_bounds {
        if b.height <= vh {
            // Level shorter than viewport → anchor to bottom edge (y = 0).
            half_vh
        } else {
            target_y.clamp(half_vh, b.height - half_vh)
        }
    } else {
        target_y
    };

    cam_tf.translation.x = cam_x;
    cam_tf.translation.y = cam_y;
}

