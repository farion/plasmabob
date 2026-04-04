use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::math::Dir2;
use bevy::prelude::*;

use crate::game::components::moving::Moving;
use crate::game::components::hitbox::PolygonHitbox;
use crate::game::components::SpawnedLevelEntity;

/// Checks 60px ahead for a platform edge. If no ground is found,
/// reverses direction immediately and resets the direction timer.
pub(crate) fn check_and_avoid_platform_edge(
    spatial_query: &SpatialQuery,
    npc_entity: Entity,
    transform: &Transform,
    moving: &mut Moving,
) {
    let npc_pos = transform.translation.xy();

    // Check position 60px ahead in current direction
    let check_ahead_x = npc_pos.x + (moving.direction * 60.0);
    let check_ahead_y = npc_pos.y - 10.0; // Start slightly below the NPC

    // Cast downward from that position to see if ground exists
    let raycast_origin = Vec2::new(check_ahead_x, check_ahead_y);
    let raycast_direction = Dir2::NEG_Y;

    // Create a filter that excludes the NPC itself
    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(npc_entity);

    // Cast downward up to 50px with solid_only=true to only hit solid objects
    let hits = spatial_query.ray_hits(raycast_origin, raycast_direction, 50.0, 10, true, &filter);

    // If no ground ahead, reverse direction immediately
    if hits.is_empty() {
        moving.direction = -moving.direction;
        moving.reset_direction_timer();
    }
}

pub(crate) fn update_sprite_flip_for_move_axis(sprite: &mut Sprite, move_axis: f32) {
    if move_axis < 0.0 && !sprite.flip_x {
        sprite.flip_x = true;
    } else if move_axis > 0.0 && sprite.flip_x {
        sprite.flip_x = false;
    }
}

pub(crate) fn direction_within_moving_bounds(direction: f32, delta_from_origin: f32, max_distance: f32) -> f32 {
    if delta_from_origin >= max_distance {
        -1.0
    } else if delta_from_origin <= -max_distance {
        1.0
    } else {
        direction
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

pub(crate) fn is_airborne_side_blocked(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &Transform,
    sprite: &Sprite,
    move_axis: f32,
) -> bool {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    let half_height = (size.y * 0.5).max(8.0);
    let edge_inset = (half_height * 0.2).clamp(6.0, 20.0);
    let sample_offsets = [
        -half_height + edge_inset,
        0.0,
        half_height - edge_inset,
    ];

    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(entity);

    let dir = if move_axis > 0.0 { Dir2::X } else { Dir2::NEG_X };
    for offset_y in sample_offsets {
        let origin = Vec2::new(transform.translation.x, transform.translation.y + offset_y);
        let hits = spatial_query.ray_hits(origin, dir, 6.0, 4, true, &filter);
        if !hits.is_empty() {
            return true;
        }
    }

    false
}

