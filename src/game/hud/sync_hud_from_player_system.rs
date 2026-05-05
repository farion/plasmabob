use bevy::prelude::*;

use crate::game::components::Health;
use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::hud::hud_state::HudState;
use crate::game::tags::PlayerTag;

pub fn sync_hud_from_player_system(
    mut hud_state: ResMut<HudState>,
    players: Query<(&Health, Option<&ControlledRangeAttack>), With<PlayerTag>>,
) {
    let Some((health, attack)) = players.iter().next() else {
        return;
    };

    let max_health = health.max.max(1) as f32;
    hud_state.health_frac = (health.current as f32 / max_health).clamp(0.0, 1.0);

    hud_state.plasma_cooldown_frac = match attack {
        Some(a) => {
            let duration = a.cooldown.duration().as_secs_f32();
            if duration <= f32::EPSILON {
                1.0
            } else {
                a.cooldown.fraction().clamp(0.0, 1.0)
            }
        }
        None => 0.0,
    };
}


