use bevy::prelude::*;

use crate::game::components::{MovingPlatform, RigidBody};

const WAYPOINT_EPSILON: f32 = 0.001;

pub fn moving_platform_system(
    time: Res<Time>,
    mut platforms: Query<(Entity, &mut Transform, &mut MovingPlatform, Option<&mut RigidBody>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (entity, mut transform, mut moving_platform, rigid_body) in &mut platforms {
        let start_position = transform.translation.truncate();

        if !moving_platform.can_move() {
            if let Some(mut rb) = rigid_body {
                rb.velocity = Vec2::ZERO;
            }
            continue;
        }

        let mut remaining = moving_platform.speed * dt;
        let mut position = start_position;

        while remaining > 0.0 {
            let Some(target) = moving_platform.waypoints.get(moving_platform.target_index).copied() else {
                break;
            };

            let to_target = target - position;
            let distance = to_target.length();

            if distance <= WAYPOINT_EPSILON {
                if !moving_platform.advance_target() {
                    break;
                }
                continue;
            }

            let step = remaining.min(distance);
            let direction = to_target / distance;
            position += direction * step;
            remaining -= step;

            if (distance - step) > WAYPOINT_EPSILON {
                break;
            }

            if !moving_platform.advance_target() {
                break;
            }
        }

        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if let Some(mut rb) = rigid_body {
            rb.velocity = (position - start_position) / dt;
        }

        // Log movement when platform actually displaced this frame
        let disp = position - start_position;
    }
}

