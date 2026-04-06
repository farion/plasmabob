use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::LevelStats;
use crate::game::components::animation::{
    AnimationState, EntityState, HitStateTimer, can_set_state,
};
use crate::game::components::player::Player;
use crate::game::systems::gameplay::helpers::{
    detect_small_step, dust_origin, ensure_dust_particle_image, spawn_dust_burst,
    update_sprite_flip_for_move_axis,
};
use crate::game::systems::systems_api::{PLAYER_JUMP_SPEED, PLAYER_MOVE_SPEED};
use crate::key_bindings::KeyBindings;

pub(crate) fn control_player(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut dust_particle_image: Local<Option<Handle<Image>>>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    spatial_query: SpatialQuery,
    mut players: Query<
        (
            Entity,
            &mut avian2d::prelude::LinearVelocity,
            Has<crate::game::systems::systems_api::Grounded>,
            &Transform,
            &mut Sprite,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&crate::game::components::health::Health>,
        ),
        With<Player>,
    >,
    mut stats: ResMut<LevelStats>,
) {
    let particle_image = ensure_dust_particle_image(&mut dust_particle_image, &mut images);

    for (entity, mut velocity, is_grounded, transform, mut sprite, mut state, hit_timer, health) in
        &mut players
    {
        if health.is_some_and(|value| value.is_dead()) {
            velocity.x = 0.0;
            continue;
        }

        let mut move_axis: f32 = 0.0;
        if keys.pressed(key_bindings.move_left) {
            move_axis -= 1.0;
        }
        if keys.pressed(key_bindings.move_right) {
            move_axis += 1.0;
        }

        let mut apply_vx = true;
        if move_axis.abs() > f32::EPSILON && !is_grounded {
            apply_vx =
                !is_airborne_side_blocked(&spatial_query, entity, transform, &sprite, move_axis);
        }

        velocity.x = if apply_vx {
            move_axis * PLAYER_MOVE_SPEED
        } else {
            0.0
        };

        if move_axis.abs() > f32::EPSILON && is_grounded {
            if let Some(step_impulse) = detect_small_step(
                &spatial_query,
                entity,
                transform,
                &sprite,
                move_axis.signum(),
                8.0,
            ) {
                velocity.y = velocity.y.max(step_impulse);
            }
        }
        update_sprite_flip_for_move_axis(&mut sprite, move_axis);

        if keys.just_pressed(key_bindings.jump) && is_grounded {
            velocity.y = PLAYER_JUMP_SPEED;
            spawn_dust_burst(
                &mut commands,
                dust_origin(transform, &sprite),
                &particle_image,
                8,
                entity.index_u32() + 1_000,
                180.0,
            );
            stats.jumps = stats.jumps.saturating_add(1);
        }

        let next_state = if !is_grounded {
            EntityState::Jump
        } else if move_axis.abs() > f32::EPSILON {
            EntityState::Walk
        } else {
            EntityState::Default
        };

        if can_set_state(&state, hit_timer, None, None, next_state) {
            state.set(next_state);
        }
    }
}

fn is_airborne_side_blocked(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &Transform,
    sprite: &Sprite,
    move_axis: f32,
) -> bool {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    let half_height = (size.y * 0.5).max(8.0);
    let edge_inset = (half_height * 0.2).clamp(6.0, 20.0);
    let sample_offsets = [-half_height + edge_inset, 0.0, half_height - edge_inset];

    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(entity);

    let dir = if move_axis > 0.0 {
        Dir2::X
    } else {
        Dir2::NEG_X
    };
    for offset_y in sample_offsets {
        let origin = Vec2::new(transform.translation.x, transform.translation.y + offset_y);
        let hits = spatial_query.ray_hits(origin, dir, 6.0, 4, true, &filter);
        if !hits.is_empty() {
            return true;
        }
    }

    false
}
