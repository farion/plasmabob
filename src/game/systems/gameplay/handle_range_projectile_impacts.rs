use avian2d::prelude::SpatialQueryFilter;
use bevy::math::Dir2;
use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HIT_STATE_SECONDS, HitStateTimer};
use crate::game::components::collision::Collision;
use crate::game::components::health::{Health, InvincibilityTimer};
use crate::game::components::player::Player;
use crate::game::systems::gameplay::types::RangeProjectile;
use crate::game::systems::presentation::health_floating;
use crate::game::systems::systems_api::PLAYER_INVINCIBILITY_SECONDS;

pub(crate) fn handle_range_projectile_impacts(
    mut commands: Commands,
    spatial_query: avian2d::prelude::SpatialQuery,
    collision_query: Query<(), With<Collision>>,
    mut player_query: Query<
        (
            Entity,
            &mut Health,
            &mut AnimationState,
            Option<&InvincibilityTimer>,
        ),
        With<Player>,
    >,
    projectiles: Query<(Entity, &RangeProjectile, &Transform)>,
) {
    for (projectile_entity, projectile, transform) in &projectiles {
        let start = projectile.previous_position;
        let end = transform.translation.truncate();
        let segment = end - start;
        let distance = segment.length();

        if distance <= f32::EPSILON {
            continue;
        }

        let Ok(direction) = Dir2::new(segment) else {
            continue;
        };

        let filter = SpatialQueryFilter {
            excluded_entities: [projectile.shooter].into_iter().collect(),
            ..default()
        };

        let hit = spatial_query
            .ray_hits(start, direction, distance, 16, true, &filter)
            .into_iter()
            .filter(|candidate| collision_query.contains(candidate.entity))
            .min_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        let Some(hit) = hit else {
            continue;
        };

        if let Ok((player_entity, mut player_health, mut player_state, invincibility)) =
            player_query.get_mut(hit.entity)
        {
            if !player_health.is_dead() && invincibility.is_none() {
                player_health.take_damage(projectile.damage);
                commands
                    .entity(player_entity)
                    .insert(health_floating::RecentHealthChange(-(projectile.damage)));
                commands
                    .entity(player_entity)
                    .insert(InvincibilityTimer::new(PLAYER_INVINCIBILITY_SECONDS));

                if !player_health.is_dead() {
                    player_state.set(EntityState::Hit);
                    commands
                        .entity(player_entity)
                        .insert(HitStateTimer::new(HIT_STATE_SECONDS, player_state.version));
                }
            }
        }

        commands.entity(projectile_entity).despawn();
    }
}

