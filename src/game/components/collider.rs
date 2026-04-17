use bevy::prelude::{Component, Vec2};

/// Collision shape used for simple physics and queries.
#[derive(Component, Debug, Clone)]
pub struct Collider {
    /// Local offset from the entity transform.
    pub offset: Vec2,
    /// The shape of the collider.
    pub shape: ColliderShape,
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Rectangle { half_extents: Vec2 },
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            offset: Vec2::ZERO,
            shape: ColliderShape::Rectangle { half_extents: Vec2::new(8.0, 8.0) },
        }
    }
}

// Note: helper methods that previously handled trigger flags and multiple
// collider shapes were removed because those variants/fields were not used.
// Keep the simple Rectangle-only runtime Collider to eliminate dead-code
// warnings and reduce surface area.
