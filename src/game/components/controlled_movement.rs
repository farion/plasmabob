use bevy::prelude::{Component, Vec2};

/// Movement component for player-controlled characters.
#[derive(Component, Debug, Clone, Copy)]
pub struct ControlledMovement {
    /// Horizontal move speed in virtual units per second.
    pub speed: f32,
    /// Jump impulse strength.
    pub jump_force: f32,
    /// Whether double jump is allowed.
    pub allow_double_jump: bool,
    /// Number of jumps performed since last grounded.
    pub jumps_performed: u8,
    /// Dash impulse strength.
    pub dash_force: f32,
    /// Maximum horizontal speed clamp (optional, 0 = no clamp).
    pub max_speed: f32,
    /// Optional facing direction: -1.0 = left, 1.0 = right
    pub facing: f32,
}

impl Default for ControlledMovement {
    fn default() -> Self {
        ControlledMovement {
            speed: 120.0,
            jump_force: 260.0,
            allow_double_jump: true,
            jumps_performed: 0,
            dash_force: 300.0,
            max_speed: 0.0,
            facing: 1.0,
        }
    }
}

impl ControlledMovement {
    /// Apply overrides from a JSON value (expected to be an object) and
    /// return the modified component. The JSON structure mirrors the
    /// `components.controlled_movement` object in entity-type JSON.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("speed").and_then(|n| n.as_f64()) {
                self.speed = v as f32;
            }
            if let Some(v) = map.get("jump_force").and_then(|n| n.as_f64()) {
                self.jump_force = v as f32;
            }
            if let Some(b) = map.get("allow_double_jump").and_then(|n| n.as_bool()) {
                self.allow_double_jump = b;
            }
            if let Some(v) = map.get("dash_force").and_then(|n| n.as_f64()) {
                self.dash_force = v as f32;
            }
            if let Some(v) = map.get("max_speed").and_then(|n| n.as_f64()) {
                self.max_speed = v as f32;
            }
            if let Some(v) = map.get("facing").and_then(|n| n.as_f64()) {
                self.facing = v as f32;
            }
        }
        self
    }
}

