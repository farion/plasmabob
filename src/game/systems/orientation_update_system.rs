use bevy::prelude::*;

use crate::game::components::orientation::{FacingDirection, Orientation};
use crate::game::components::{AutoMovement, ControlledMovement, RigidBody, Collider};
use crate::game::setup::flip_utils::flip_entity_preserve_collider;

/// Updates the `Orientation` component for entities that have a movement component.
///
/// - Entities with `ControlledMovement`: facing is derived from `RigidBody.velocity.x`.
/// - Entities with `AutoMovement` (but not `ControlledMovement`): facing is derived from
///   `AutoMovement.direction.x`.
///
/// Facing is only updated when the relevant axis value is non-zero; the last known direction
/// is preserved while the entity is standing still.
pub fn orientation_update_system(
    mut controlled: Query<(&RigidBody, &mut Orientation, &mut Sprite, Option<&mut Transform>, Option<&mut Collider>), With<ControlledMovement>>,
    mut auto_mover: Query<(&AutoMovement, &mut Orientation, &mut Sprite, Option<&mut Transform>, Option<&mut Collider>), Without<ControlledMovement>>,
) {
    for (rb, mut orientation, mut sprite, mut maybe_tr, mut maybe_col) in &mut controlled {
        let prev = orientation.facing;
        if rb.velocity.x < -f32::EPSILON {
            orientation.facing = FacingDirection::Left;
        } else if rb.velocity.x > f32::EPSILON {
            orientation.facing = FacingDirection::Right;
        }
        // If facing changed, adjust sprite flip and preserve collider world centre.
        if orientation.facing != prev {
            if let (Some(col), Some(tr)) = (maybe_col.as_mut(), maybe_tr.as_mut()) {
                let desired_flip = matches!(orientation.facing, FacingDirection::Left);
                flip_entity_preserve_collider(tr, col, &mut sprite, desired_flip);
            } else {
                // No collider/transform available to preserve; just set flip.
                sprite.flip_x = matches!(orientation.facing, FacingDirection::Left);
            }
        }
        // Velocity near zero: keep current facing direction.
    }

    for (auto_mov, mut orientation, mut sprite, mut maybe_tr, mut maybe_col) in &mut auto_mover {
        let prev = orientation.facing;
        if auto_mov.direction.x < -f32::EPSILON {
            orientation.facing = FacingDirection::Left;
        } else if auto_mov.direction.x > f32::EPSILON {
            orientation.facing = FacingDirection::Right;
        }
        if orientation.facing != prev {
            if let (Some(col), Some(tr)) = (maybe_col.as_mut(), maybe_tr.as_mut()) {
                let desired_flip = matches!(orientation.facing, FacingDirection::Left);
                flip_entity_preserve_collider(tr, col, &mut sprite, desired_flip);
            } else {
                sprite.flip_x = matches!(orientation.facing, FacingDirection::Left);
            }
        }
    }
}

