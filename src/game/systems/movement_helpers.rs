use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::math::Dir2;
use bevy::prelude::*;


pub(crate) fn update_sprite_flip_for_move_axis(sprite: &mut Sprite, move_axis: f32) {
    if move_axis < 0.0 && !sprite.flip_x {
        sprite.flip_x = true;
    } else if move_axis > 0.0 && sprite.flip_x {
        sprite.flip_x = false;
    }
}


/// If a small vertical step is detected ahead of the entity, nudge the entity up
/// so it can continue moving instead of getting stuck. The tolerance parameter
/// is the maximum vertical step (in pixels) we will climb.
pub(crate) fn detect_small_step(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &Transform,
    _sprite: &Sprite,
    direction: f32,
    max_step: f32,
) -> Option<f32> {
    let foot_offset = -10.0;
    let probe_x = transform.translation.x + (direction * 8.0);
    let probe_y = transform.translation.y + foot_offset;

    let origin_current = Vec2::new(transform.translation.x, probe_y);
    let origin_ahead = Vec2::new(probe_x, probe_y);

    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(entity);

    let hits_current = spatial_query.ray_hits(origin_current, Dir2::NEG_Y, 40.0, 8, true, &filter);
    let hits_ahead = spatial_query.ray_hits(origin_ahead, Dir2::NEG_Y, 40.0, 8, true, &filter);

    if hits_current.is_empty() || hits_ahead.is_empty() {
        return None;
    }

    let current_min = hits_current.iter().map(|h| h.distance).fold(f32::INFINITY, f32::min);
    let ahead_min = hits_ahead.iter().map(|h| h.distance).fold(f32::INFINITY, f32::min);

    let current_ground_y = origin_current.y - current_min;
    let ahead_ground_y = origin_ahead.y - ahead_min;

    if ahead_ground_y > current_ground_y {
        let step = ahead_ground_y - current_ground_y;
        if step > 0.5 && step <= max_step {
            // Return a small upward velocity to climb the step; scale to avoid high jumps.
            return Some((step + 8.0).min(220.0));
        }
    }

    None
}
