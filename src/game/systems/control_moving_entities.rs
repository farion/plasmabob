use avian2d::prelude::{LinearVelocity, SpatialQuery};
use bevy::prelude::*;

use crate::game::components::moving::Moving;
use crate::game::components::npc::Npc;
use crate::game::components::health::Health;
use crate::game::components::animation::{AnimationState, FightStateTimer, HitStateTimer, can_set_state, EntityState};

use crate::game::systems::common::movement_helpers::{
    check_and_avoid_platform_edge,
    update_sprite_flip_for_move_axis,
    direction_within_moving_bounds,
    detect_small_step,
};

use crate::game::view_api::MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN;

pub(crate) fn control_moving_entities(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut entities: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            &mut Sprite,
            &mut Moving,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&FightStateTimer>,
            Option<&Health>,
        ),
        With<Npc>,
    >,
) {
    for (entity, transform, mut velocity, mut sprite, mut moving, mut state, hit_timer, fight_timer, health) in &mut entities {
        if health.is_some_and(|value| value.is_dead()) {
            velocity.x = 0.0;
            continue;
        }

        moving.direction_change_timer.tick(time.delta());

        if moving.direction_change_timer.just_finished() {
            moving.direction = if moving.next_random_unit() < 0.5 { -1.0 } else { 1.0 };
            moving.reset_direction_timer();

            // Occasionally change speed when changing direction.
            if moving.next_random_unit() < 0.3 {
                moving.randomize_speed();
            }
        }

        let delta_from_origin = transform.translation.x - moving.origin_x;
        moving.direction = direction_within_moving_bounds(moving.direction, delta_from_origin, MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN);

        // Check 60px ahead for platform edge - reverse direction BEFORE falling
        check_and_avoid_platform_edge(&spatial_query, entity, transform, moving);

        // Try to step up small bumps so moving NPCs don't get stuck on tiny geometry.
        if let Some(step_amount) = detect_small_step(&spatial_query, entity, transform, &sprite, moving.direction, 8.0) {
            // Apply a small upward impulse proportional to the step amount so the
            // physics/rigidbody can resolve the step-over.
            velocity.y = step_amount.max(velocity.y);
        }

        velocity.x = moving.direction * moving.speed;
        update_sprite_flip_for_move_axis(&mut sprite, velocity.x);

        let next_state = if velocity.x.abs() > f32::EPSILON {
            EntityState::Walk
        } else {
            EntityState::Default
        };

        if can_set_state(&state, hit_timer, fight_timer, next_state) {
            state.set(next_state);
        }
    }
}

