use bevy::prelude::*;
use bevy::time::Virtual;

use crate::game::hud::hud_state::HudState;

pub fn tick_level_time_system(
    mut hud_state: ResMut<HudState>,
    mut stats: ResMut<crate::LevelStats>,
    time: Res<Time<Virtual>>,
) {
    stats.total_time_seconds += time.delta_secs();
    hud_state.level_seconds = stats.total_time_seconds;
    hud_state.score = stats.score;
}

