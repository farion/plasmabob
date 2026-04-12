use bevy::prelude::{Component, Vec2};

/// Gravity component applied to entities affected by gravity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Gravity {
    /// Gravity scale applied to the global gravity vector.
    pub scale: f32,
    /// Whether the entity is currently considered grounded.
    pub grounded: bool,
    /// Additional per-entity acceleration (e.g. from slopes or explosions).
    pub extra_accel: Vec2,
}

impl Default for Gravity {
    fn default() -> Self {
        Gravity { scale: 1.0, grounded: false, extra_accel: Vec2::ZERO }
    }
}

