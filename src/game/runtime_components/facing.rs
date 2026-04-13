use bevy::prelude::{Component, Vec2};

/// Stores the last non-zero facing direction for controlled entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct Facing {
    pub direction: Vec2,
}

impl Default for Facing {
    fn default() -> Self {
        Self { direction: Vec2::X }
    }
}
