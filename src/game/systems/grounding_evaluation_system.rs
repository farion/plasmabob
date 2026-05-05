use avian2d::prelude::Gravity as WorldGravity;
use bevy::prelude::*;

use crate::game::components::{Gravity, RigidBody};
use crate::game::runtime_components::GroundingState;

const SUPPORT_THRESHOLD: f32 = 0.95;
const GROUND_EXIT_HYSTERESIS_SEC: f32 = 0.10;

pub fn grounding_evaluation_system(
    time: Res<Time>,
    world_gravity: Res<WorldGravity>,
    mut entities: Query<(&mut Gravity, &RigidBody, &mut GroundingState, &Transform)>,
    mut jump_queue: ResMut<crate::game::gfx::jump::JumpParticlesQueue>,
) {
    let dt = time.delta_secs();
    let gravity_strength = Vec2::new(world_gravity.0.x, world_gravity.0.y).length();

    for (mut gravity, rigid_body, mut grounding_state, transform) in &mut entities {
        let required_support = rigid_body.mass.max(0.0) * gravity_strength * SUPPORT_THRESHOLD;
        let was_grounded = gravity.grounded;
        // Note: since we included Transform in the query, we'll instead destructure below.
        let support_force =
            rigid_body.mass.max(0.0) * gravity_strength * grounding_state.support_normal_sum_y;

        if support_force >= required_support && grounding_state.support_normal_sum_y > 0.0 {
            gravity.grounded = true;
            grounding_state.unsupported_time = 0.0;
            if !was_grounded {
                // Land transition: send particle event at feet position
                // Use support_velocity to infer horizontal direction bias
                let origin = transform.translation.truncate();
                let horizontal_dir = grounding_state.support_velocity.x.signum();
                let seed_base = origin.x.to_bits().wrapping_mul(31) ^ origin.y.to_bits().wrapping_mul(131) ^ 0xBEEF_C0DEu32;
                jump_queue.0.push(crate::game::gfx::jump::JumpParticlesEvent::Land { origin, horizontal_dir, seed_base });
            }
        } else {
            grounding_state.unsupported_time += dt;
            if grounding_state.unsupported_time >= GROUND_EXIT_HYSTERESIS_SEC {
                gravity.grounded = false;
                grounding_state.support_velocity = Vec2::ZERO;
                grounding_state.support_entity = None;
            }
        }
    }
}
