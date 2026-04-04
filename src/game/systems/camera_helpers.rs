use bevy::prelude::*;

use crate::game::view_api::{ActiveLevelBounds, PLAYER_SCREEN_X_ANCHOR};

/// Update the camera Transform X coordinate so the player stays at the
/// configured screen anchor while respecting optional level bounds.
pub(crate) fn update_camera_x(
    camera_transform: &mut Transform,
    player_x: f32,
    viewport_width: f32,
    level_bounds: Option<ActiveLevelBounds>,
) {
    camera_transform.translation.x = camera_x_for_player(player_x, viewport_width, level_bounds);
}

fn camera_x_for_player(
    player_x: f32,
    viewport_width: f32,
    level_bounds: Option<ActiveLevelBounds>,
) -> f32 {
    let target_x = player_x + (0.5 - PLAYER_SCREEN_X_ANCHOR) * viewport_width;

    match level_bounds {
        Some(bounds) => clamp_camera_x_to_bounds(target_x, viewport_width, bounds),
        None => target_x,
    }
}

fn clamp_camera_x_to_bounds(target_x: f32, viewport_width: f32, bounds: ActiveLevelBounds) -> f32 {
    let min_camera_x = bounds.left + (viewport_width * 0.5);
    let max_camera_x = bounds.right - (viewport_width * 0.5);

    if min_camera_x > max_camera_x {
        bounds.center_x()
    } else {
        target_x.clamp(min_camera_x, max_camera_x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_player_at_40_percent_of_screen_width_without_bounds() {
        let camera_x = camera_x_for_player(100.0, 1000.0, None);

        assert_eq!(camera_x, 200.0);
    }

    #[test]
    fn clamps_camera_at_left_level_edge() {
        let bounds = ActiveLevelBounds {
            left: -400.0,
            right: 4184.0,
            bottom: -300.0,
            top: 724.0,
        };

        let camera_x = camera_x_for_player(-200.0, 800.0, Some(bounds));

        assert_eq!(camera_x, 0.0);
    }

    #[test]
    fn centers_camera_when_level_is_smaller_than_viewport() {
        let bounds = ActiveLevelBounds {
            left: -400.0,
            right: 200.0,
            bottom: -300.0,
            top: 724.0,
        };

        let camera_x = camera_x_for_player(50.0, 800.0, Some(bounds));

        assert_eq!(camera_x, -100.0);
    }
}


