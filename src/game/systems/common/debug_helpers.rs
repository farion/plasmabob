use bevy::prelude::*;

pub(crate) fn toggle_hitbox_lines(debug_settings: &mut crate::DebugRenderSettings) {
    debug_settings.show_hitbox_lines = !debug_settings.show_hitbox_lines;
}

pub(crate) fn build_stats_text(
    health: Option<&crate::game::components::health::Health>,
    damage: Option<&crate::game::components::health::Damage>,
    plasma: Option<&crate::game::components::player::PlasmaAttack>,
    state: Option<&crate::game::components::animation::AnimationState>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(hp) = health {
        parts.push(format!("HP {}/{}", hp.current, hp.max));
    }

    if let Some(p) = plasma {
        parts.push(format!("ATK-RNG {:.0}", p.range));
        parts.push(format!("DMG {}", p.damage));
    } else if let Some(dmg) = damage {
        parts.push(format!("DMG {}", dmg.0));
    }

    if let Some(state) = state {
        parts.push(format!("STATE {}", state.current.animation_key()));
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::animation::EntityState;
    use crate::game::components::health::Health;

    #[test]
    fn toggles_hitbox_debug_lines() {
        let mut settings = crate::DebugRenderSettings {
            show_hitbox_lines: false,
            show_overlay: false,
        };

        toggle_hitbox_lines(&mut settings);
        assert!(settings.show_hitbox_lines);

        toggle_hitbox_lines(&mut settings);
        assert!(!settings.show_hitbox_lines);
    }

    #[test]
    fn build_stats_text_includes_state_line() {
        let state = crate::game::components::animation::AnimationState {
            current: EntityState::Fight,
            version: 0,
        };

        let text = build_stats_text(None, None, None, Some(&state));

        assert_eq!(text, "STATE fight");
    }

    #[test]
    fn build_stats_text_includes_state_with_other_stats() {
        let health = Health { current: 7, max: 10 };
        let state = crate::game::components::animation::AnimationState {
            current: EntityState::Walk,
            version: 3,
        };

        let text = build_stats_text(Some(&health), None, None, Some(&state));

        assert!(text.contains("HP 7/10"));
        assert!(text.contains("STATE walk"));
    }
}

