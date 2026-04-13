use bevy::prelude::*;

use crate::game::components::{ControlledMovement, Gravity, RigidBody};
use crate::game::runtime_components::Facing;
use crate::game::tags::PlayerTag;
use crate::helper::key_bindings::KeyBindings;

pub fn player_control_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut players: Query<
        (
            Entity,
            &ControlledMovement,
            &mut Gravity,
            &mut RigidBody,
            Option<&mut Facing>,
        ),
        With<PlayerTag>,
    >,
) {
    for (entity, movement, mut gravity, mut rigid_body, facing) in &mut players {
        let mut axis = 0.0;
        if keyboard.pressed(key_bindings.move_left) {
            axis -= 1.0;
        }
        if keyboard.pressed(key_bindings.move_right) {
            axis += 1.0;
        }

        rigid_body.velocity.x = axis * movement.speed;

        if axis.abs() > f32::EPSILON {
            let direction = Vec2::new(axis.signum(), 0.0);
            if let Some(mut facing) = facing {
                facing.direction = direction;
            } else {
                commands.entity(entity).insert(Facing { direction });
            }
        } else if facing.is_none() {
            commands.entity(entity).insert(Facing::default());
        }

        if keyboard.just_pressed(key_bindings.jump) && gravity.grounded {
            rigid_body.velocity.y = movement.jump_force;
            gravity.grounded = false;
        }
    }
}

