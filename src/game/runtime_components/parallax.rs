use bevy::prelude::{Component, Resource, Vec2};

/// Per-entity parallax metadata.
///
/// `base_position` is the world-space position captured when the level starts.
/// `factor` controls relative scroll speed against camera motion:
/// - `1.0` = normal camera scroll
/// - `< 1.0` = slower than camera
/// - `> 1.0` = faster than camera
#[derive(Component, Debug, Clone, Copy)]
pub struct Parallax {
    pub base_position: Vec2,
    pub factor: f32,
}

/// Camera position at parallax initialization time.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ParallaxCameraOrigin(pub Vec2);

