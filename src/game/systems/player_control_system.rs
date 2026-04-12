use bevy::prelude::*;

use crate::game::components::{ControlledMovement, Gravity, RigidBody};
use crate::game::tags::PlayerTag;
use crate::helper::key_bindings::KeyBindings;

pub fn player_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut players: Query<
        (&ControlledMovement, &mut Gravity, &mut RigidBody),
        With<PlayerTag>,
    >,
) {
    for (movement, mut gravity, mut rigid_body) in &mut players {
        let mut axis = 0.0;
        if keyboard.pressed(key_bindings.move_left) {
            axis -= 1.0;
        }
        if keyboard.pressed(key_bindings.move_right) {
            axis += 1.0;
        }

        rigid_body.velocity.x = axis * movement.speed;

        if keyboard.just_pressed(key_bindings.jump) && gravity.grounded {
            rigid_body.velocity.y = movement.jump_force;
            gravity.grounded = false;
        }
    }
}

