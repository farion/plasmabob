use bevy::prelude::*;

use crate::game::components::collider::Collider;

/// Flip an entity's sprite to `desired_flip` while preserving the collider's
/// world-centre. This toggles `sprite.flip_x`, negates `collider.offset.x` and
/// shifts `transform.translation.x` so that the collider world X centre
/// remains unchanged.
pub fn flip_entity_preserve_collider(transform: &mut Transform, collider: &mut Collider, sprite: &mut bevy::sprite::Sprite, desired_flip: bool) {
    if sprite.flip_x == desired_flip {
        return;
    }
    let old_offset = collider.offset.x;
    let old_tr_x = transform.translation.x;
    let old_world_cx = old_tr_x + old_offset;

    // Apply flip and mirror collider offset
    sprite.flip_x = desired_flip;
    collider.offset.x = -old_offset;

    // Set transform so new world centre equals old world centre
    transform.translation.x = old_world_cx - collider.offset.x;
}

/// Given a new collider (in sprite-local coordinates), adjust its offset so
/// that the collider's world X centre remains equal to the previous world
/// centre (derived from `old_col_offset_x` and `transform`), and mirror the
/// new collider if `is_flipped` is true.
pub fn adjust_new_collider_preserve_world_center(transform: &Transform, old_col_offset_x: f32, new_col: &mut Collider, is_flipped: bool) {
    let cur_tr_x = transform.translation.x;
    let old_world_cx = cur_tr_x + old_col_offset_x;

    if is_flipped {
        new_col.offset.x = -new_col.offset.x;
    }
    new_col.offset.x = old_world_cx - cur_tr_x;
}


