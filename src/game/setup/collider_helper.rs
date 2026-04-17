use bevy::prelude::Vec2;

use crate::game::components::{Collider, ColliderShape};

/// Build a [`Collider`] from a `collider_box` polygon (pixel coords in sprite-image
/// space, origin top-left).  Falls back to the full sprite rectangle when `collider_box`
/// is `None` or has fewer than two points.
///
/// Conversion from image space → entity-local Bevy space:
///   local_x = pixel_x − sprite_w / 2          (shift origin to centre)
///   local_y = sprite_h / 2 − pixel_y          (flip Y axis)
pub fn build_collider_from_box(
    collider_box: Option<&[[f32; 2]]>,
    sprite_w: f32,
    sprite_h: f32,
) -> Collider {
    if let Some(pts) = collider_box {
        if pts.len() >= 2 {
            let min_x = pts.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
            let max_x = pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
            let min_y = pts.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
            let max_y = pts.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

            let half_w = (max_x - min_x) / 2.0;
            let half_h = (max_y - min_y) / 2.0;

            let cx_img = (min_x + max_x) / 2.0;
            let cy_img = (min_y + max_y) / 2.0;
            let offset_x = cx_img - sprite_w / 2.0;
            let offset_y = sprite_h / 2.0 - cy_img;

            return Collider {
                offset: Vec2::new(offset_x, offset_y),
                shape: ColliderShape::Rectangle {
                    half_extents: Vec2::new(half_w, half_h),
                },
            };
        }
    }

    // Fallback: full sprite bounding box centred at origin.
    Collider {
        offset: Vec2::ZERO,
        shape: ColliderShape::Rectangle {
            half_extents: Vec2::new(sprite_w / 2.0, sprite_h / 2.0),
        },
    }
}

