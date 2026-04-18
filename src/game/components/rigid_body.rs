use bevy::prelude::{Component, Vec2};

/// Simple rigid body storing dynamic physics state.
#[derive(Component, Debug, Clone)]
pub struct RigidBody {
    /// Linear velocity in virtual world units per second.
    pub velocity: Vec2,
    /// Mass of the body. A mass of 0.0 marks a static/immovable body.
    pub mass: f32,
    /// Linear drag applied each second.
    pub linear_damp: f32,
    /// Restitution (bounciness) used in collisions.
    pub restitution: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            velocity: Vec2::ZERO,
            mass: 1.0,
            linear_damp: 0.0,
            restitution: 0.0,
        }
    }
}

impl RigidBody {
    pub fn is_static(&self) -> bool {
        self.mass <= 0.0
    }
}

// JSON-based override removed; prefer typed `override_from_config`.

crate::impl_override_from_config!(RigidBody, crate::game::level::configs::RigidBodyConfig,
    pick_vec2 => [velocity],
    pick_f32 => [mass, linear_damp, restitution],
);

