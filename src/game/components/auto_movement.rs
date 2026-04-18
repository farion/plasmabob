use bevy::prelude::{Component, Vec2};

/// Component for simple autonomous movement (used by enemies, platforms, etc.).
#[derive(Component, Debug, Clone, Copy)]
pub struct AutoMovement {
    /// Unit direction the entity will try to move in. Use Vec2::ZERO to stop.
    pub direction: Vec2,
    /// Speed in virtual units per second.
    pub speed: f32,
    /// Whether the movement is currently active.
    pub enabled: bool,
}

impl Default for AutoMovement {
    fn default() -> Self {
        AutoMovement { direction: Vec2::ZERO, speed: 0.0, enabled: true }
    }
}

crate::impl_override_from_config!(AutoMovement, crate::game::level::configs::AutoMovementConfig,
    pick_vec2 => [direction],
    pick_f32 => [speed],
    pick_bool => [enabled],
);
