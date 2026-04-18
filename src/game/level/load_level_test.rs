use bevy::prelude::*;

use crate::game::level::loader::load_level_from_asset;
use crate::helper::active_character::ActiveCharacter;

// Integration-like unit test for `load_level_from_asset` using the real assets
// folder. The test constructs a minimal Bevy `App` so we can obtain a working
// `AssetServer` that reads from the project's `assets/` directory.

#[test]
fn load_viridara_level_from_assets() {
    // Build a minimal Bevy app that provides the AssetServer resource.
    let mut app = App::new();
    // Use the minimal plugin bundle plus the AssetPlugin so the AssetServer is
    // present and configured to use the filesystem asset source.
    app.add_plugins(MinimalPlugins).add_plugin(bevy::asset::AssetPlugin::default());

    // Obtain the AssetServer from the app world.
    let asset_server = app.world.resource::<AssetServer>().clone();

    // Path relative to the assets/ directory in the repository root.
    let asset_path = "worlds/auralis/viridara_level1.json";

    let loaded = load_level_from_asset(&asset_server, asset_path, ActiveCharacter::Bob)
        .expect("should load level");

    // Basic sanity checks: asset path should be recorded and a LevelDefinition present.
    assert_eq!(loaded.asset_path.as_deref(), Some(asset_path));
    assert!(loaded.level.is_some());

    // There should be at least one entity type loaded for this world.
    assert!(!loaded.entity_types.is_empty(), "expected entity types to be loaded");
}

