use bevy::prelude::*;
use bevy::time::Virtual;

use crate::game::hud::hud_state::HudState;

pub fn tick_level_time_system(mut hud_state: ResMut<HudState>, time: Res<Time<Virtual>>) {
    hud_state.level_seconds += time.delta_secs();
}

