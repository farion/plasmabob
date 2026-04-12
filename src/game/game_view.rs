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
            cleanup_cached_level,
        )
        .add_plugins(crate::game::setup::SetupPlugin)
        .add_plugins(crate::game::systems::SystemsPlugin);
    }
}

/// Loads the level specified by the global `LevelSelection` resource using
/// `game::level::loader::load_level_from_asset` and inserts/overwrites the
/// `CachedLevelDefinition` resource so game systems can consume it.
fn load_selected_level(
    asset_server: Res<AssetServer>,
    level_selection: Res<crate::LevelSelection>,
    mut commands: Commands,
) {
    // Defer to the existing loader which performs all IO and parsing and
    // returns a `CachedLevelDefinition` on success.
    match crate::game::level::loader::load_level_from_asset(&asset_server, level_selection.asset_path()) {
        Ok(cached) => {
            commands.insert_resource(cached);
            tracing::info!(level = %level_selection.asset_path(), "Loaded level into CachedLevelDefinition");
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
