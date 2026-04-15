use bevy::prelude::*;

pub mod cleanup_level;
pub mod collider_helper;
pub mod entity_type_assets;
pub mod follow_camera;
pub mod setup_background;
pub mod setup_canvas;
pub mod spawn_entities;

pub use entity_type_assets::{EntityTypeAsset, EntityTypeAssets, StateAssets};

use crate::app_model::AppState;
use crate::game::game_view::GameSetupSet;
use crate::game::gfx::fire_shoot::{preload_fire_particle_image, cleanup_fire_particle_image};
use crate::game::gfx::plasma_shoot::{preload_plasma_particle_image, cleanup_plasma_particle_image};
use crate::game::systems::apply_parallax_system::apply_parallax_system;
use crate::game::systems::init_parallax_system::init_parallax_system;

/// Plugin that registers the initial level-setup systems and the exit cleanup.
///
/// Systems run in `OnEnter(AppState::GameView)` inside [`GameSetupSet::Setup`]
/// (which is ordered after [`GameSetupSet::LoadLevel`] in `GameViewPlugin`):
///   1. `setup_canvas`      – position the camera at the player spawn
///   2. `setup_background`  – spawn tiled background sprites
///   3. `spawn_entities`    – spawn all level entities
///c
/// An `Update` system (`follow_camera`) runs every frame while in `GameView`
/// to keep the camera tracking the player at the configured screen anchor.
/// A `PostUpdate` resize system reapplies the same camera rules immediately
/// when the window size changes.
///
/// On `OnExit(AppState::GameView)` the `cleanup_game_entities` system despawns
/// all [`GameEntity`]-tagged entities.
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::GameView),
            (
                // Preload particle images at level start so visuals are available
                // for the current level only.
                preload_plasma_particle_image,
                preload_fire_particle_image,
                setup_canvas::setup_canvas,
                setup_background::setup_background,
                spawn_entities::spawn_entities,
                init_parallax_system,
            )
                .chain()
                .in_set(GameSetupSet::Setup),
        )
        .add_systems(
            Update,
            (
                follow_camera::follow_camera,
                apply_parallax_system.after(follow_camera::follow_camera),
            )
                .run_if(in_state(AppState::GameView)),
        )
        .add_systems(
            PostUpdate,
            follow_camera::follow_camera_on_resize.run_if(in_state(AppState::GameView)),
        )
        .add_systems(
            OnExit(AppState::GameView),
            (
                cleanup_level::cleanup_game_entities,
                // Remove the preloaded particle images and free the generated
                // textures so they don't persist between levels.
                cleanup_plasma_particle_image,
                cleanup_fire_particle_image,
            ),
        );
    }
}
