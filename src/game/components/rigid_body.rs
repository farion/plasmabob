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

impl RigidBody {
    /// Apply overrides from `components.rigid_body` JSON object.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("mass").and_then(|n| n.as_f64()) {
                self.mass = v as f32;
            }
            if let Some(v) = map.get("linear_damp").and_then(|n| n.as_f64()) {
                self.linear_damp = v as f32;
            }
            if let Some(v) = map.get("restitution").and_then(|n| n.as_f64()) {
                self.restitution = v as f32;
            }
            if let Some(arr) = map.get("velocity").and_then(|n| n.as_array()) {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        self.velocity = Vec2::new(x as f32, y as f32);
                    }
                }
            }
        }
        self
    }
}

