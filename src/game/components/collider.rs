use bevy::prelude::{Component, Vec2};
use crate::game::level::types::StateConfig;

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

impl Collider {
    /// Apply overrides from `components.collider` JSON object.
    /// Supported keys:
    /// - `offset`: [x, y]
    /// - `is_trigger`: bool
    /// - `shape`: object with either `rectangle` { half_extents: [w,h] } or `circle` { radius }
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(arr) = map.get("offset").and_then(|v| v.as_array()) {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        self.offset = Vec2::new(x as f32, y as f32);
                    }
                }
            }
            if let Some(b) = map.get("is_trigger").and_then(|v| v.as_bool()) {
                self.is_trigger = b;
            }
            if let Some(shape_val) = map.get("shape") {
                if let serde_json::Value::Object(shape_map) = shape_val {
                    if let Some(rect) = shape_map.get("rectangle") {
                        if let Some(hx) = rect.get("half_extents").and_then(|v| v.as_array()) {
                            if hx.len() >= 2 {
                                if let (Some(w), Some(h)) = (hx[0].as_f64(), hx[1].as_f64()) {
                                    self.shape = ColliderShape::Rectangle { half_extents: Vec2::new(w as f32, h as f32) };
                                }
                            }
                        }
                    } else if let Some(circle) = shape_map.get("circle") {
                        if let Some(r) = circle.get("radius").and_then(|v| v.as_f64()) {
                            self.shape = ColliderShape::Circle { radius: r as f32 };
                        }
                    }
                }
            }
        }
        self
    }
}

impl Collider {
    /// Build a `Collider` from a state's `collider_box` (pixel coords in image
    /// space, origin top-left). Falls back to the full sprite rectangle when the
    /// collider box is absent. Mirrors previous `build_collider` helper.
    pub fn from_state_config(state_cfg: &StateConfig, sprite_w: f32, sprite_h: f32) -> Collider {
        if let Some(pts) = &state_cfg.collider_box {
            if pts.len() >= 2 {
                let min_x = pts.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
                let max_x = pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
                let min_y = pts.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
                let max_y = pts.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

                let half_w = (max_x - min_x) / 2.0;
                let half_h = (max_y - min_y) / 2.0;

                // Centre of the box in image space → entity-local Bevy space.
                let cx_img = (min_x + max_x) / 2.0;
                let cy_img = (min_y + max_y) / 2.0;
                let offset_x = cx_img - sprite_w / 2.0;
                let offset_y = sprite_h / 2.0 - cy_img;

                return Collider {
                    offset: Vec2::new(offset_x, offset_y),
                    shape: ColliderShape::Rectangle {
                        half_extents: Vec2::new(half_w, half_h),
                    },
                    is_trigger: false,
                };
            }
        }

        // Fallback: full sprite bounding box centred at origin.
        Collider {
            offset: Vec2::ZERO,
            shape: ColliderShape::Rectangle {
                half_extents: Vec2::new(sprite_w / 2.0, sprite_h / 2.0),
            },
            is_trigger: false,
        }
    }
}

