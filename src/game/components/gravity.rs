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

impl Gravity {
    /// Apply overrides from `components.gravity` JSON object.
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(v) = map.get("scale").and_then(|n| n.as_f64()) {
                self.scale = v as f32;
            }
            if let Some(arr) = map.get("extra_accel").and_then(|n| n.as_array()) {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        self.extra_accel = Vec2::new(x as f32, y as f32);
                    }
                }
            }
        }
        self
    }
}

