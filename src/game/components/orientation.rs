use bevy::prelude::{Component, Vec2};
// OrientationConfig is referenced by the override macro using its full path; the local use was unused and removed

/// Left/right facing direction of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    Right,
}

impl Default for FacingDirection {
    fn default() -> Self {
        FacingDirection::Right
    }
}

/// Orientation component tracking the facing direction and surface alignment of an entity.
///
/// - `facing` defaults to `Right`.
/// - `surface_alignment` defaults to `Vec2::ZERO` (aligned with world up).
///
/// Both values can be overridden via the level JSON under `components.orientation`.
#[derive(Component, Debug, Clone, Copy)]
pub struct Orientation {
    /// Left or right facing direction.
    pub facing: FacingDirection,
    /// Surface alignment as a 2D vector (e.g. ground normal). Defaults to Vec2::ZERO.
    pub surface_alignment: Vec2,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation {
            facing: FacingDirection::Right,
            surface_alignment: Vec2::ZERO,
        }
    }
}


crate::impl_override_from_config!(Orientation, crate::game::level::configs::OrientationConfig,
    pick_facing => [facing],
    pick_vec2 => [surface_alignment],
);

