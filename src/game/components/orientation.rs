use bevy::prelude::{Component, Vec2};

/// Left/right facing direction of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    Right,
}

impl Default for FacingDirection {
    fn default() -> Self {
        FacingDirection::Right
    }
}

/// Orientation component tracking the facing direction and surface alignment of an entity.
///
/// - `facing` defaults to `Right`.
/// - `surface_alignment` defaults to `Vec2::ZERO` (aligned with world up).
///
/// Both values can be overridden via the level JSON under `components.orientation`.
#[derive(Component, Debug, Clone, Copy)]
pub struct Orientation {
    /// Left or right facing direction.
    pub facing: FacingDirection,
    /// Surface alignment as a 2D vector (e.g. ground normal). Defaults to Vec2::ZERO.
    pub surface_alignment: Vec2,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation {
            facing: FacingDirection::Right,
            surface_alignment: Vec2::ZERO,
        }
    }
}

impl Orientation {
    /// Apply overrides from a JSON `components.orientation` object.
    ///
    /// Supported keys:
    /// - `facing`: `"left"` or `"right"` (default: `"right"`)
    /// - `surface_alignment`: `[x, y]` array (default: `[0.0, 0.0]`)
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(s) = map.get("facing").and_then(|v| v.as_str()) {
                self.facing = match s.to_ascii_lowercase().as_str() {
                    "left" => FacingDirection::Left,
                    _ => FacingDirection::Right,
                };
            }
            if let Some(arr) = map.get("surface_alignment").and_then(|v| v.as_array()) {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        self.surface_alignment = Vec2::new(x as f32, y as f32);
                    }
                }
            }
        }
        self
    }
}

