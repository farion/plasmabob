use avian2d::prelude::{LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::game::components::moving::Moving;
use crate::game::components::npc::Npc;
use crate::game::components::health::Health;
use crate::game::components::animation::{AnimationState, FightStateTimer, HitStateTimer, can_set_state, EntityState};
use crate::game::systems::gameplay::helpers::{detect_small_step, update_sprite_flip_for_move_axis};

use crate::game::systems::systems_api::MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN;

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
        check_and_avoid_platform_edge(&spatial_query, entity, transform, &mut *moving);

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

/// Checks 60px ahead for a platform edge. If no ground is found,
/// reverses direction immediately and resets the direction timer.
fn check_and_avoid_platform_edge(
    spatial_query: &SpatialQuery,
    npc_entity: Entity,
    transform: &Transform,
    moving: &mut Moving,
) {
    let npc_pos = transform.translation.xy();

    // Check position 60px ahead in current direction
    let check_ahead_x = npc_pos.x + (moving.direction * 60.0);
    let check_ahead_y = npc_pos.y - 10.0; // Start slightly below the NPC

    // Cast downward from that position to see if ground exists
    let raycast_origin = Vec2::new(check_ahead_x, check_ahead_y);
    let raycast_direction = Dir2::NEG_Y;

    // Create a filter that excludes the NPC itself
    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(npc_entity);

    // Cast downward up to 50px with solid_only=true to only hit solid objects
    let hits = spatial_query.ray_hits(raycast_origin, raycast_direction, 50.0, 10, true, &filter);

    // If no ground ahead, reverse direction immediately
    if hits.is_empty() {
        moving.direction = -moving.direction;
        moving.reset_direction_timer();
    }
}

fn direction_within_moving_bounds(direction: f32, delta_from_origin: f32, max_distance: f32) -> f32 {
    if delta_from_origin >= max_distance {
        -1.0
    } else if delta_from_origin <= -max_distance {
        1.0
    } else {
        direction
    }
}


