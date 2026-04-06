use bevy::prelude::*;

use crate::game::systems::systems_api::{ActiveLevelBounds, PLAYER_SCREEN_X_ANCHOR};

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

pub(crate) fn health_fraction(current: i32, max: i32) -> f32 {
    if max <= 0 {
        return 0.0;
    }

    (current as f32 / max as f32).clamp(0.0, 1.0)
}

pub(crate) fn percentage_text(fraction: f32) -> String {
    format!("{}%", (fraction * 100.0).round() as i32)
}

const HUD_BAR_INNER_WIDTH: f32 = 260.0 - 2.0 * 2.0; // match hud constants
pub(crate) fn filled_bar_width(fraction: f32) -> f32 {
    HUD_BAR_INNER_WIDTH * fraction.clamp(0.0, 1.0)
}

pub(crate) fn cooldown_fraction(
    plasma_attack: &crate::game::components::player::PlasmaAttack,
) -> f32 {
    plasma_attack.cooldown.fraction().clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::player::PlasmaAttack;

    #[test]
    fn clamps_health_fraction() {
        assert_eq!(health_fraction(50, 100), 0.5);
        assert_eq!(health_fraction(200, 100), 1.0);
        assert_eq!(health_fraction(-10, 100), 0.0);
    }

    #[test]
    fn handles_zero_or_negative_max_health() {
        assert_eq!(health_fraction(10, 0), 0.0);
        assert_eq!(health_fraction(10, -5), 0.0);
    }

    #[test]
    fn formats_percentage_text() {
        assert_eq!(percentage_text(1.0), "100%");
        assert_eq!(percentage_text(0.755), "76%");
        assert_eq!(percentage_text(0.0), "0%");
    }

    #[test]
    fn computes_filled_bar_width_inside_white_frame() {
        assert_eq!(filled_bar_width(1.0), HUD_BAR_INNER_WIDTH);
        assert_eq!(filled_bar_width(0.5), HUD_BAR_INNER_WIDTH * 0.5);
        assert_eq!(filled_bar_width(-1.0), 0.0);
        assert_eq!(filled_bar_width(2.0), HUD_BAR_INNER_WIDTH);
    }

    #[test]
    fn tracks_plasma_cooldown_fraction() {
        let mut plasma_attack = PlasmaAttack::new(400.0, 10);
        assert_eq!(cooldown_fraction(&plasma_attack), 1.0);

        plasma_attack.cooldown.reset();
        assert_eq!(cooldown_fraction(&plasma_attack), 0.0);

        plasma_attack
            .cooldown
            .tick(std::time::Duration::from_secs_f32(0.25));
        assert_eq!(cooldown_fraction(&plasma_attack), 0.5);
    }

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
