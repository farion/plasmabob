use bevy::prelude::*;

use crate::game::gameplay::components::GameEntity;
use crate::game::level::types::CachedLevelDefinition;

/// Spawns the level background sprite, sized to fill the entire level bounds.
///
/// The background asset path is resolved from (in priority order):
/// 1. `level.background`
/// 2. `level.terrain.background`
///
/// If neither is present the system exits silently.
pub fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cached: Res<CachedLevelDefinition>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("setup_background: no level loaded, skipping background setup");
        return;
    };

    // Resolve background asset path as an owned String to avoid lifetime issues
    // with the borrowed Res<> when calling asset_server.load().
    let bg_path: Option<String> = level
        .background
        .clone()
        .or_else(|| level.terrain.as_ref()?.background.clone());

    let Some(bg_path) = bg_path else {
        tracing::debug!("setup_background: no background path defined for this level");
        return;
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    // Center the background sprite over the full level area.
    let center_x = bounds.width / 2.0;
    let center_y = bounds.height / 2.0;

    // Z = 0 places the background behind all gameplay entities.
    commands.spawn((
        Sprite {
            image: asset_server.load(bg_path.clone()),
            custom_size: Some(Vec2::new(bounds.width, bounds.height)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y, 0.0),
        GameEntity,
    ));

    tracing::info!(path = %bg_path, "Background sprite spawned");
}
