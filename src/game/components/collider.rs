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

// JSON-based override removed; prefer typed `override_from_config` where applicable.

impl Collider {
    pub fn override_from_config(mut self, entity_cfg: Option<&Collider>, level_cfg: Option<&Collider>) -> Self {
        self.offset = entity_cfg.map(|c| c.offset).or(level_cfg.map(|c| c.offset)).unwrap_or(self.offset);
        self.is_trigger = entity_cfg.map(|c| c.is_trigger).or(level_cfg.map(|c| c.is_trigger)).unwrap_or(self.is_trigger);
        // Prefer entity_cfg.shape -> level_cfg.shape -> existing
        self.shape = entity_cfg.and_then(|c| Some(c.shape.clone())).or(level_cfg.and_then(|c| Some(c.shape.clone()))).unwrap_or_else(|| self.shape.clone());
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

