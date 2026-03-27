use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer, can_set_state};
use crate::game::components::health::Health;
use crate::game::components::moving::Moving;
use crate::game::components::npc::Npc;

use super::MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN;

pub(super) fn control_moving_entities(
	time: Res<Time>,
	mut entities: Query<
		(
			&Transform,
			&mut LinearVelocity,
			&mut Sprite,
			&mut Moving,
			&mut AnimationState,
			Option<&HitStateTimer>,
			Option<&Health>,
		),
		With<Npc>,
	>,
) {
	for (transform, mut velocity, mut sprite, mut moving, mut state, hit_timer, health) in &mut entities {
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

		velocity.x = moving.direction * moving.speed;
		update_sprite_flip_for_move_axis(&mut sprite, velocity.x);

		let next_state = if velocity.x.abs() > f32::EPSILON {
			EntityState::Walk
		} else {
			EntityState::Default
		};

		if can_set_state(&state, hit_timer, next_state) {
			state.set(next_state);
		}
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

