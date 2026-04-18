use avian2d::prelude::Gravity as WorldGravity;
use bevy::prelude::*;

use crate::game::components::{Gravity, RigidBody};

pub fn gravity_integration_system(
    time: Res<Time>,
    world_gravity: Res<WorldGravity>,
    mut entities: Query<(&Gravity, &mut RigidBody)>,
) {
    let dt = time.delta_secs();
    let gravity_vec = Vec2::new(world_gravity.0.x, world_gravity.0.y);

    for (gravity, mut rigid_body) in &mut entities {
        if rigid_body.is_static() {
            continue;
        }

        let accel = gravity_vec * gravity.scale + gravity.extra_accel;
        rigid_body.velocity += accel * dt;

        if rigid_body.linear_damp > 0.0 {
            let damp = (1.0 - rigid_body.linear_damp * dt).clamp(0.0, 1.0);
            rigid_body.velocity *= damp;
        }
    }
}

