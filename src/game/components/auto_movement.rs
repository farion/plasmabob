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

impl AutoMovement {
    /// Apply JSON overrides from `components.auto_movement` object.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("speed").and_then(|n| n.as_f64()) {
                self.speed = v as f32;
            } else if let (Some(min), Some(max)) = (
                map.get("min_speed").and_then(|n| n.as_f64()),
                map.get("max_speed").and_then(|n| n.as_f64()),
            ) {
                self.speed = (((min + max) / 2.0) as f32).max(0.0);
            }
            if let Some(b) = map.get("enabled").and_then(|n| n.as_bool()) {
                self.enabled = b;
            }
            if let Some(arr) = map.get("direction").and_then(|n| n.as_array()) {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        self.direction = Vec2::new(x as f32, y as f32);
                    }
                }
            }
        }
        self
    }
}

