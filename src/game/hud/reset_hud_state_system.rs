use bevy::prelude::*;

use crate::game::hud::hud_state::HudState;

pub fn reset_hud_state_system(mut hud_state: ResMut<HudState>) {
    *hud_state = HudState::default();
}
