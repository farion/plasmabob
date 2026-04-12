use bevy::prelude::{Component, Vec2};

/// Collision shape used for simple physics and queries.
#[derive(Component, Debug, Clone)]
pub struct Collider {
    /// Local offset from the entity transform.
    pub offset: Vec2,
    /// The shape of the collider.
    pub shape: ColliderShape,
    /// If true, this collider does not produce physical collisions and only sends overlap events.
    pub is_trigger: bool,
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Rectangle { half_extents: Vec2 },
    Circle { radius: f32 },
    Polygon { points: Vec<Vec2> },
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            offset: Vec2::ZERO,
            shape: ColliderShape::Rectangle { half_extents: Vec2::new(8.0, 8.0) },
            is_trigger: false,
        }
    }
}

