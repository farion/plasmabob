use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, FightStateTimer, HitStateTimer, FIGHT_STATE_SECONDS, HIT_STATE_SECONDS, can_set_state};
use crate::game::components::health::{Damage, Health, InvincibilityTimer};
use crate::game::components::hostile::Hostile;
use crate::game::components::player::Player;
use crate::game::systems::presentation::health_floating;
use crate::game::systems::systems_api::PLAYER_INVINCIBILITY_SECONDS;

pub(crate) fn apply_hostile_contact_damage(
    mut commands: Commands,
    _time: Res<Time>,
    mut hostiles: Query<(
        Entity,
        &Damage,
        Option<&mut Health>,
        &Transform,
        &mut AnimationState,
        Option<&HitStateTimer>,
        Option<&FightStateTimer>,
        Option<&crate::game::components::LevelEntityType>,
    ), (With<Hostile>, Without<Player>)>,
    mut player_query: Query<(
        Entity,
        &avian2d::prelude::CollidingEntities,
        &Transform,
        &avian2d::prelude::LinearVelocity,
        &mut Health,
        &mut AnimationState,
        Option<&crate::game::components::player::PlasmaAttack>,
    ), (With<Player>, Without<InvincibilityTimer>, Without<Hostile>)>,
) {
    for (
        player_entity,
        colliding_entities,
        _player_transform,
        _player_velocity,
        mut player_health,
        mut player_state,
        _plasma_attack_opt,
    ) in &mut player_query
    {
        if player_health.is_dead() {
            continue;
        }

        for &colliding_entity in colliding_entities.0.iter() {
            if let Ok((
                hostile_entity,
                damage,
                _hostile_health_opt,
                _hostile_transform,
                mut hostile_state,
                hostile_hit_timer,
                hostile_fight_timer,
                _level_entity_type,
            )) = hostiles.get_mut(colliding_entity)
            {
                // Apply contact damage immediately (stomping removed).
                player_health.take_damage(damage.0);
                // Mark entity for floating health text (negative = damage)
                commands.entity(player_entity).insert(health_floating::RecentHealthChange(-(damage.0)));
                commands.entity(player_entity).insert(InvincibilityTimer::new(PLAYER_INVINCIBILITY_SECONDS));

                if !player_health.is_dead() {
                    player_state.set(EntityState::Hit);
                    commands.entity(player_entity).insert(HitStateTimer::new(HIT_STATE_SECONDS, player_state.version));
                }

                if can_set_state(&hostile_state, hostile_hit_timer, hostile_fight_timer, EntityState::Fight) {
                    hostile_state.set(EntityState::Fight);
                    commands.entity(hostile_entity).insert(FightStateTimer::new(FIGHT_STATE_SECONDS));
                }

                info!(
                    "Player took {} damage from hostile - HP: {}/{}",
                    damage.0, player_health.current, player_health.max
                );
                break;
            }
        }
    }
}

