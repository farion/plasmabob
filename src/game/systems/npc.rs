use avian2d::prelude::{LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy::math::Dir2;

use crate::game::components::animation::{AnimationState, EntityState, FightStateTimer, HitStateTimer, can_set_state};
use crate::game::components::health::Health;
use crate::game::components::moving::Moving;
use crate::game::components::npc::Npc;

use super::MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN;

pub(super) fn control_moving_entities(
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
		moving.direction = direction_within_moving_bounds(moving.direction, delta_from_origin);

		// Check 60px ahead for platform edge - reverse direction BEFORE falling
		check_and_avoid_platform_edge(&spatial_query, entity, transform, &mut moving);


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

fn update_sprite_flip_for_move_axis(sprite: &mut Sprite, move_axis: f32) {
	if move_axis < 0.0 && !sprite.flip_x {
		sprite.flip_x = true;
	} else if move_axis > 0.0 && sprite.flip_x {
		sprite.flip_x = false;
	}
}

fn direction_within_moving_bounds(direction: f32, delta_from_origin: f32) -> f32 {
	if delta_from_origin >= MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN {
		-1.0
	} else if delta_from_origin <= -MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN {
		1.0
	} else {
		direction
	}
}

/// If a small vertical step is detected ahead of the entity, nudge the entity up
/// so it can continue moving instead of getting stuck. The tolerance parameter
/// is the maximum vertical step (in pixels) we will climb.
fn detect_small_step(
	spatial_query: &SpatialQuery,
	entity: Entity,
	transform: &Transform,
	_sprite: &Sprite,
	direction: f32,
	max_step: f32,
) -> Option<f32> {
	let foot_offset = -10.0;
	let probe_x = transform.translation.x + (direction * 8.0);
	let probe_y = transform.translation.y + foot_offset;

	let origin_current = Vec2::new(transform.translation.x, probe_y);
	let origin_ahead = Vec2::new(probe_x, probe_y);

	let mut filter = SpatialQueryFilter::default();
	filter.excluded_entities.insert(entity);

	let hits_current = spatial_query.ray_hits(origin_current, Dir2::NEG_Y, 40.0, 8, true, &filter);
	let hits_ahead = spatial_query.ray_hits(origin_ahead, Dir2::NEG_Y, 40.0, 8, true, &filter);

	if hits_current.is_empty() || hits_ahead.is_empty() {
		return None;
	}

	let current_min = hits_current.iter().map(|h| h.distance).fold(f32::INFINITY, f32::min);
	let ahead_min = hits_ahead.iter().map(|h| h.distance).fold(f32::INFINITY, f32::min);

	let current_ground_y = origin_current.y - current_min;
	let ahead_ground_y = origin_ahead.y - ahead_min;

	if ahead_ground_y > current_ground_y {
		let step = ahead_ground_y - current_ground_y;
		if step > 0.5 && step <= max_step {
			// Return a small upward velocity to climb the step; scale to avoid high jumps.
			return Some((step + 8.0).min(220.0));
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn moving_entities_turn_back_at_right_patrol_limit() {
		let direction = direction_within_moving_bounds(1.0, 500.0);

		assert_eq!(direction, -1.0);
	}

	#[test]
	fn moving_entities_turn_back_at_left_patrol_limit() {
		let direction = direction_within_moving_bounds(-1.0, -500.0);

		assert_eq!(direction, 1.0);
	}
}

