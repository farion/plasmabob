use bevy::prelude::*;

use crate::game::components::orientation::{FacingDirection, Orientation};
use crate::game::components::{AutoMovement, ControlledMovement, RigidBody};

/// Updates the `Orientation` component for entities that have a movement component.
///
/// - Entities with `ControlledMovement`: facing is derived from `RigidBody.velocity.x`.
/// - Entities with `AutoMovement` (but not `ControlledMovement`): facing is derived from
///   `AutoMovement.direction.x`.
///
/// Facing is only updated when the relevant axis value is non-zero; the last known direction
/// is preserved while the entity is standing still.
pub fn orientation_update_system(
    mut controlled: Query<(&RigidBody, &mut Orientation), With<ControlledMovement>>,
    mut auto_mover: Query<
        (&AutoMovement, &mut Orientation),
        Without<ControlledMovement>,
    >,
) {
    for (rb, mut orientation) in &mut controlled {
        if rb.velocity.x < -f32::EPSILON {
            orientation.facing = FacingDirection::Left;
        } else if rb.velocity.x > f32::EPSILON {
            orientation.facing = FacingDirection::Right;
        }
        // Velocity near zero: keep current facing direction.
    }

    for (auto_mov, mut orientation) in &mut auto_mover {
        if auto_mov.direction.x < -f32::EPSILON {
            orientation.facing = FacingDirection::Left;
        } else if auto_mov.direction.x > f32::EPSILON {
            orientation.facing = FacingDirection::Right;
        }
    }
}

