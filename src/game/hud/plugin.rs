use bevy::prelude::*;

use crate::app_model::AppState;
use crate::game::hud::cleanup_hud_system::cleanup_hud_system;
use crate::game::hud::hud_state::HudState;
use crate::game::hud::reset_hud_state_system::reset_hud_state_system;
use crate::game::hud::spawn_hud_system::spawn_hud_system;
use crate::game::hud::sync_hud_from_player_system::sync_hud_from_player_system;
use crate::game::hud::tick_level_time_system::tick_level_time_system;
use crate::game::hud::update_hud_bars_system::update_hud_bars_system;
use crate::game::hud::update_hud_lives_system::update_hud_lives_system;
use crate::game::hud::update_hud_text_system::update_hud_text_system;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudState>()
            .add_systems(
                OnEnter(AppState::GameView),
                (reset_hud_state_system, spawn_hud_system).chain(),
            )
            .add_systems(OnExit(AppState::GameView), cleanup_hud_system)
            .add_systems(
                Update,
                (
                    sync_hud_from_player_system,
                    tick_level_time_system,
                    update_hud_bars_system,
                    update_hud_text_system,
                    update_hud_lives_system,
                )
                    .chain()
                    .run_if(in_state(AppState::GameView)),
            );
    }
}


