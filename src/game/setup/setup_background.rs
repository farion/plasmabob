use bevy::prelude::*;

use crate::game::gameplay::components::GameEntity;
use crate::game::level::types::CachedLevelDefinition;
use crate::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

/// Width of each background tile in world units (= virtual viewport width).
const BG_TILE_W: f32 = VIRTUAL_WIDTH;
/// Height of each background tile in world units (= virtual viewport height).
const BG_TILE_H: f32 = VIRTUAL_HEIGHT;

/// Spawns tiled background sprites that cover the entire level bounds.
///
/// Each tile is `BG_TILE_W × BG_TILE_H` world units (1024 × 768), matching
/// the virtual viewport so one tile fills the screen exactly.  The number of
/// tiles in each axis is `ceil(level_dim / tile_dim)`, guaranteeing full
/// coverage without gaps.
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

    // Resolve background asset path.
    let bg_path: Option<String> = level
        .background
        .clone()
        .or_else(|| level.terrain.as_ref()?.background.clone());

    let Some(bg_path) = bg_path else {
        tracing::debug!("setup_background: no background path defined for this level");
        return;
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    let tiles_x = (bounds.width / BG_TILE_W).ceil() as u32;
    let tiles_y = (bounds.height / BG_TILE_H).ceil() as u32;
    let tile_count = tiles_x * tiles_y;

    // Load the image handle once and clone the cheap Arc-based handle per tile.
    let image: Handle<Image> = asset_server.load(bg_path.clone());

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            // Tile origin is bottom-left; sprite centre is offset by half tile size.
            let center_x = tx as f32 * BG_TILE_W + BG_TILE_W / 2.0;
            let center_y = ty as f32 * BG_TILE_H + BG_TILE_H / 2.0;

            // Z = 0: background tiles are behind all gameplay entities (positive z).
            // Must be > -1, which is the near clipping plane with camera at z=999.
            commands.spawn((
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::new(BG_TILE_W, BG_TILE_H)),
                    ..default()
                },
                Transform::from_xyz(center_x, center_y, 0.0),
                GameEntity,
            ));
        }
    }

    tracing::info!(
        path = %bg_path,
        tiles_x,
        tiles_y,
        tile_count,
        "Background tiled across level"
    );
}
