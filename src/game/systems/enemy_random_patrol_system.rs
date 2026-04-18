use bevy::prelude::*;

use crate::game::components::{AutoMovement, RigidBody, StateMachine};
use crate::game::runtime_components::PatrolState;
use crate::game::tags::EnemyTag;

const PATROL_MIN_INTERVAL_SEC: f32 = 0.8;
const PATROL_MAX_INTERVAL_SEC: f32 = 2.4;
const DEFAULT_PATROL_SPEED: f32 = 80.0;

pub fn enemy_random_patrol_system(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<
        (
            Entity,
            &mut AutoMovement,
            &mut RigidBody,
            Option<&mut PatrolState>,
            Option<&StateMachine>,
        ),
        With<EnemyTag>,
    >,
) {
    let dt = time.delta_secs();

    for (entity, mut auto_movement, mut rigid_body, patrol_state, state_machine) in &mut enemies {
        if state_machine.is_some_and(|sm| sm.is_non_interactive()) {
            auto_movement.direction = Vec2::ZERO;
            rigid_body.velocity.x = 0.0;
            continue;
        }

        let Some(mut patrol_state) = patrol_state else {
            commands.entity(entity).insert(PatrolState::from_entity(entity));
            continue;
        };

        if !auto_movement.enabled {
            rigid_body.velocity.x = 0.0;
            continue;
        }

        patrol_state.timer -= dt;
        if patrol_state.timer <= 0.0 {
            let rand_value = patrol_state.next_rand();
            patrol_state.direction = if rand_value < 0.4 {
                -1.0
            } else if rand_value > 0.6 {
                1.0
            } else {
                0.0
            };

            let interval_rand = patrol_state.next_rand();
            patrol_state.timer = PATROL_MIN_INTERVAL_SEC
                + (PATROL_MAX_INTERVAL_SEC - PATROL_MIN_INTERVAL_SEC) * interval_rand;
        }

        auto_movement.direction = Vec2::new(patrol_state.direction, 0.0);
        let speed = if auto_movement.speed > 0.0 {
            auto_movement.speed
        } else {
            DEFAULT_PATROL_SPEED
        };
        rigid_body.velocity.x = auto_movement.direction.x * speed;
    }
}

