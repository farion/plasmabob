use bevy::prelude::*;
use std::collections::HashMap;

use crate::app_model::AppState;

/// System-set labels used to order the two phases of GameView initialisation.
///
/// `LoadLevel` runs first and populates `CachedLevelDefinition`.
/// `Setup` runs second and spawns the visual scene from that resource.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSetupSet {
    /// Load level JSON and entity-type data into `CachedLevelDefinition`.
    LoadLevel,
    /// Spawn camera position, background, and level entities.
    Setup,
}

pub struct GameViewPlugin;

// When the Game view is entered we must ensure the selected level asset is
// loaded and made available as the `CachedLevelDefinition` resource so the
// rest of the game systems can spawn entities and play music.
impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        // Enforce ordering: all Setup systems run after LoadLevel.
        app.configure_sets(
            OnEnter(AppState::GameView),
            GameSetupSet::Setup.after(GameSetupSet::LoadLevel),
        )
        .add_systems(
            OnEnter(AppState::GameView),
            load_selected_level.in_set(GameSetupSet::LoadLevel),
        )
        .add_systems(
            OnExit(AppState::GameView),
            (cleanup_cached_level, reset_main_camera, reset_music_to_menu),
        )
        .add_plugins(crate::game::hud::HudPlugin)
        .add_plugins(crate::game::setup::SetupPlugin)
        .add_plugins(crate::game::systems::SystemsPlugin);
    }
}

/// Loads the level specified by the global `LevelSelection` resource using
/// `game::level::loader::load_level_from_asset` and inserts/overwrites the
/// `CachedLevelDefinition` resource so game systems can consume it.
///
/// When coming from `LoadView` the `CachedLevelDefinition` is already present;
/// this system skips reloading in that case to avoid redundant IO.
fn load_selected_level(
    asset_server: Res<AssetServer>,
    level_selection: Res<crate::LevelSelection>,
    existing: Option<Res<crate::game::level::types::CachedLevelDefinition>>,
    mut commands: Commands,
    mut music_request: ResMut<crate::helper::music::MusicRequest>,
) {
    // If LoadView already populated the resource, nothing to do.
    if existing.is_some() {
        tracing::debug!("load_selected_level: CachedLevelDefinition already present, skipping");
        return;
    }
    // Defer to the existing loader which performs all IO and parsing and
    // returns a `CachedLevelDefinition` on success.
    match crate::game::level::loader::load_level_from_asset(&asset_server, level_selection.asset_path()) {
        Ok(cached) => {
            // Extract music playlist (if any) before moving `cached` into resources.
            let music_opt = cached.level.as_ref().and_then(|l| l.music.clone());

            commands.insert_resource(cached);
            tracing::info!(level = %level_selection.asset_path(), "Loaded level into CachedLevelDefinition");

            if let Some(music) = music_opt {
                if !music.is_empty() {
                    // Request playlist playback via the global music player.
                    music_request.0 = Some(music);
                }
            }
        }
        Err(err) => {
            // Log the error first, then move it into the cached resource so
            // callers (UI or systems) can inspect it.
            tracing::error!(level = %level_selection.asset_path(), error = ?err, "Failed to load level");

            // If loading fails, create a CachedLevelDefinition that contains the
            // error so UI systems (or the GameView) can display it.
            let empty = crate::game::level::types::CachedLevelDefinition {
                asset_path: Some(level_selection.asset_path().to_string()),
                level: None,
                entity_types: HashMap::new(),
                error: Some(err),
            };
            commands.insert_resource(empty);
        }
    }
}

/// Clean up the cached level resource when leaving the Game view to avoid
/// stale data when returning later.
fn cleanup_cached_level(mut commands: Commands) {
    // Remove the resource if present.
    commands.remove_resource::<crate::game::level::types::CachedLevelDefinition>();
}

/// Reset the main camera transform when leaving the Game view so UI/menu
/// views that expect the camera at the origin render correctly.
fn reset_main_camera(mut cameras: Query<&mut Transform, With<crate::MainCamera>>) {
    for mut tf in cameras.iter_mut() {
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
        tf.translation.z = 0.0;
        tf.rotation = Default::default();
    }
}

/// Restore global music to the menu track when leaving the Game view.
fn reset_music_to_menu(
    mut music_request: ResMut<crate::helper::music::MusicRequest>,
    active_character: Res<crate::helper::active_character::ActiveCharacter>,
) {
    music_request.0 = Some(vec![active_character.menu_music_path().to_string()]);
}

