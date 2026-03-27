use avian2d::prelude::{Collider, LinearVelocity, ShapeCaster, ShapeHits};
use bevy::ecs::query::Has;
use bevy::math::Dir2;
use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer, can_set_state};
use crate::game::components::health::Health;
use crate::game::components::hitbox::PrecomputedPlayerHitbox;
use crate::game::components::moving::Moving;
use crate::game::components::player::Player;

use super::{Grounded, PLAYER_JUMP_SPEED, PLAYER_MOVE_SPEED};

pub(super) fn configure_player_controller(
    mut commands: Commands,
    mut players: Query<(Entity, &PrecomputedPlayerHitbox), (With<Player>, Without<ShapeCaster>)>,
) {
    for (player, precomputed_hitbox) in &mut players {
        commands.entity(player).insert(
            ShapeCaster::new(
                precomputed_hitbox.ground_caster(false),
                Vec2::ZERO,
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(8.0),
        );
    }
}

pub(super) fn update_grounded(
    mut commands: Commands,
    players: Query<(Entity, &ShapeHits), (With<Player>, With<ShapeCaster>)>,
) {
    for (player, hits) in &players {
        let is_grounded = !hits.is_empty();
        if is_grounded {
            commands.entity(player).insert(Grounded);
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
}

pub(super) fn control_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<
        (
            &mut LinearVelocity,
            Has<Grounded>,
            &mut Sprite,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&Health>,
        ),
        With<Player>,
    >,
) {
    for (mut velocity, is_grounded, mut sprite, mut state, hit_timer, health) in &mut players {
        if health.is_some_and(|value| value.is_dead()) {
            velocity.x = 0.0;
            continue;
        }

        let mut move_axis = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) {
            move_axis -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            move_axis += 1.0;
        }

        velocity.x = move_axis * PLAYER_MOVE_SPEED;
        update_sprite_flip_for_move_axis(&mut sprite, move_axis);

        if keys.just_pressed(KeyCode::ArrowUp) && is_grounded {
            velocity.y = PLAYER_JUMP_SPEED;
        }

        let next_state = if !is_grounded {
            EntityState::Jump
        } else if move_axis.abs() > f32::EPSILON {
            EntityState::Walk
        } else {
            EntityState::Default
        };

        if can_set_state(&state, hit_timer, next_state) {
            state.set(next_state);
        }
    }
}

pub(super) fn sync_player_hitbox_orientation(
    mut players: Query<
        (&Sprite, &PrecomputedPlayerHitbox, &mut Collider, Option<&mut ShapeCaster>),
        (Or<(With<Player>, With<Moving>)>, Changed<Sprite>),
    >,
) {
    for (sprite, precomputed_hitbox, mut collider, shape_caster) in &mut players {
        *collider = precomputed_hitbox.collider(sprite.flip_x);

        if let Some(mut shape_caster) = shape_caster {
            shape_caster.shape = precomputed_hitbox.ground_caster(sprite.flip_x);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::hitbox::PolygonHitbox;

    #[test]
    fn flips_sprite_when_moving_left() {
        let mut sprite = Sprite::default();

        update_sprite_flip_for_move_axis(&mut sprite, -1.0);

        assert!(sprite.flip_x);
    }

    #[test]
    fn unflips_sprite_when_moving_right() {
        let mut sprite = Sprite {
            flip_x: true,
            ..default()
        };

        update_sprite_flip_for_move_axis(&mut sprite, 1.0);

        assert!(!sprite.flip_x);
    }

    #[test]
    fn keeps_last_sprite_direction_while_idle() {
        let mut sprite = Sprite {
            flip_x: true,
            ..default()
        };

        update_sprite_flip_for_move_axis(&mut sprite, 0.0);

        assert!(sprite.flip_x);
    }

    #[test]
    fn mirrors_hitbox_points_when_sprite_is_flipped() {
        let polygon_hitbox = PolygonHitbox {
            points: vec![
                Vec2::new(-2.0, -1.0),
                Vec2::new(3.0, -1.0),
                Vec2::new(1.0, 4.0),
            ],
        };

        let mirrored = polygon_hitbox.effective_points(true);

        assert_eq!(
            mirrored,
            vec![
                Vec2::new(-1.0, 4.0),
                Vec2::new(-3.0, -1.0),
                Vec2::new(2.0, -1.0),
            ]
        );
    }

}



