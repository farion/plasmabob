use bevy::prelude::*;

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

pub(crate) fn cooldown_fraction(plasma_attack: &crate::game::components::player::PlasmaAttack) -> f32 {
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
}

