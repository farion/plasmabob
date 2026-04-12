use bevy::prelude::*;

pub mod cleanup_level;
pub mod follow_camera;
pub mod setup_background;
pub mod setup_canvas;
pub mod spawn_entities;

use crate::app_model::AppState;
use crate::game::game_view::GameSetupSet;

/// Plugin that registers the initial level-setup systems and the exit cleanup.
///
/// Systems run in `OnEnter(AppState::GameView)` inside [`GameSetupSet::Setup`]
/// (which is ordered after [`GameSetupSet::LoadLevel`] in `GameViewPlugin`):
///   1. `setup_canvas`      – position the camera at the player spawn
///   2. `setup_background`  – spawn tiled background sprites
///   3. `spawn_entities`    – spawn all level entities
///
/// An `Update` system (`follow_camera`) runs every frame while in `GameView`
/// to keep the camera tracking the player at the configured screen anchor.
///
/// On `OnExit(AppState::GameView)` the `cleanup_game_entities` system despawns
/// all [`GameEntity`]-tagged entities.
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::GameView),
            (
                setup_canvas::setup_canvas,
                setup_background::setup_background,
                spawn_entities::spawn_entities,
            )
                .chain()
                .in_set(GameSetupSet::Setup),
        )
        .add_systems(
            Update,
            follow_camera::follow_camera.run_if(in_state(AppState::GameView)),
        )
        .add_systems(
            OnExit(AppState::GameView),
            cleanup_level::cleanup_game_entities,
        );
    }
}

