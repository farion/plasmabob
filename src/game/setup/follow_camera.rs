use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window, WindowResized};

use crate::game::level::types::LevelBounds;
use crate::game::tags::PlayerTag;
use crate::game::level::types::CachedLevelDefinition;
use crate::{MainCamera, PLAYER_SCREEN_X, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

fn clamp_camera_axis(target: f32, neg_extent: f32, pos_extent: f32, level_size: f32) -> f32 {
    let min_center = neg_extent;
    let max_center = level_size - pos_extent;

    if max_center <= min_center {
        // Level is smaller than the viewport on this axis: lock to the left/bottom edge.
        min_center
    } else {
        target.clamp(min_center, max_center)
    }
}

fn camera_extents(projection: &Projection, window: Option<&Window>) -> (f32, f32, f32, f32) {
    let (vw, vh) = if let Some(window) = window {
        let ww = window.width();
        let wh = window.height();
        if ww > 0.0 && wh > 0.0 {
            let aspect = ww / wh;
            let min_aspect = VIRTUAL_WIDTH / VIRTUAL_HEIGHT;
            if aspect >= min_aspect {
                (VIRTUAL_HEIGHT * aspect, VIRTUAL_HEIGHT)
            } else {
                (VIRTUAL_WIDTH, VIRTUAL_WIDTH / aspect)
            }
        } else {
            (VIRTUAL_WIDTH, VIRTUAL_HEIGHT)
        }
    } else {
        match projection {
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
        }
    };

    match projection {
        Projection::Orthographic(ortho) => {
            let left = vw * ortho.viewport_origin.x;
            let right = vw - left;
            let bottom = vh * ortho.viewport_origin.y;
            let top = vh - bottom;
            (left, right, bottom, top)
        }
        _ => (vw / 2.0, vw / 2.0, vh / 2.0, vh / 2.0),
    }
}

pub(crate) fn camera_center_for_player(
    player_x: f32,
    player_y: f32,
    projection: &Projection,
    window: Option<&Window>,
    level_bounds: Option<&LevelBounds>,
) -> Vec2 {
    let (left_extent, right_extent, bottom_extent, top_extent) = camera_extents(projection, window);
    let vw = left_extent + right_extent;

    // With viewport_origin (0.5, 0.5): camera_x = player_x + (0.5 - anchor) * viewport_width.
    let target_x = player_x + (0.5 - PLAYER_SCREEN_X) * vw;
    let target_y = player_y;

    let cam_x = if let Some(bounds) = level_bounds {
        clamp_camera_axis(target_x, left_extent, right_extent, bounds.width)
    } else {
        target_x
    };

    let cam_y = if let Some(bounds) = level_bounds {
        clamp_camera_axis(target_y, bottom_extent, top_extent, bounds.height)
    } else {
        target_y
    };

    Vec2::new(cam_x, cam_y)
}

fn apply_camera_follow(
    player_tf: &Transform,
    cam_tf: &mut Transform,
    projection: &Projection,
    window: Option<&Window>,
    cached: Option<&CachedLevelDefinition>,
) {
    let level_bounds = cached
        .and_then(|c| c.level.as_ref())
        .and_then(|l| l.bounds.as_ref());

    let cam = camera_center_for_player(
        player_tf.translation.x,
        player_tf.translation.y,
        projection,
        window,
        level_bounds,
    );

    cam_tf.translation.x = cam.x;
    cam_tf.translation.y = cam.y;
}

/// Every frame: move the camera so the player appears at
/// (PLAYER_SCREEN_X, 0.5) = (30 %, 50 %) of the viewport,
/// unless doing so would show outside the level bounds.
///
/// The actual viewport size in world units is derived from the current
/// window aspect (AutoMin behaviour), so the system is resize-safe.
pub fn follow_camera(
    players: Query<&Transform, (With<PlayerTag>, Without<MainCamera>)>,
    mut cameras: Query<
        (&mut Transform, &Projection),
        (With<MainCamera>, Without<PlayerTag>),
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    cached: Option<Res<CachedLevelDefinition>>,
) {
    let Ok(player_tf) = players.single() else {
        return;
    };
    let Ok((mut cam_tf, projection)) = cameras.single_mut() else {
        return;
    };
    let window = windows.single().ok();

    apply_camera_follow(player_tf, &mut cam_tf, projection, window, cached.as_deref());
}

pub fn follow_camera_on_resize(
    mut resized_events: MessageReader<WindowResized>,
    players: Query<&Transform, (With<PlayerTag>, Without<MainCamera>)>,
    mut cameras: Query<(&mut Transform, &Projection), (With<MainCamera>, Without<PlayerTag>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cached: Option<Res<CachedLevelDefinition>>,
) {
    // Read all resize events this frame; if none, exit early.
    let mut seen = false;
    for ev in resized_events.read() {
        // Print to stdout so it's visible even without a tracing subscriber.
        println!("[follow_camera_on_resize] WindowResized event: {}x{}", ev.width, ev.height);
        tracing::info!(width = ev.width, height = ev.height, "WindowResized event received");
        seen = true;
    }
    if !seen {
        return;
    }

    let Ok(player_tf) = players.single() else {
        println!("[follow_camera_on_resize] no player found");
        return;
    };
    let Ok((mut cam_tf, projection)) = cameras.single_mut() else {
        println!("[follow_camera_on_resize] no main camera found");
        return;
    };
    let window = windows.single().ok();

    // Compute camera centre using the same helper so behaviour matches setup and runtime.
    let level_bounds = cached
        .as_deref()
        .and_then(|c| c.level.as_ref())
        .and_then(|l| l.bounds.as_ref());

    let cam = camera_center_for_player(
        player_tf.translation.x,
        player_tf.translation.y,
        projection,
        window,
        level_bounds,
    );

    println!("[follow_camera_on_resize] Applying camera pos x={:.2} y={:.2}", cam.x, cam.y);
    tracing::info!(x = cam.x, y = cam.y, "Applying camera position after resize");
    cam_tf.translation.x = cam.x;
    cam_tf.translation.y = cam.y;
}

