use bevy::prelude::*;

use crate::game::components::LevelEntityType;
use crate::game::components::animation::{
    AnimationState, EntityState, HIT_STATE_SECONDS, HitStateTimer, MELEE_ATTACK_STATE_SECONDS,
    MeleeAttackStateTimer, can_set_state,
};
use crate::game::components::health::{Health, InvincibilityTimer};
use crate::game::components::melee_attack::MeleeAttack;
use crate::game::components::player::{PlasmaAttack, Player};
use crate::game::systems::presentation::health_floating;
use crate::game::systems::systems_api::PLAYER_INVINCIBILITY_SECONDS;

pub(crate) fn apply_meele_attack_contact_damage(
    mut commands: Commands,
    _time: Res<Time>,
    mut enemy: Query<(
            Entity,
            &MeleeAttack,
            &Transform,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&MeleeAttackStateTimer>,
            Option<&LevelEntityType>,
        ), (With<MeleeAttack>, Without<Player>)>,
    mut player_query: Query<
        (
            Entity,
            &avian2d::prelude::CollidingEntities,
            &Transform,
            &avian2d::prelude::LinearVelocity,
            &mut Health,
            &mut AnimationState,
            Option<&PlasmaAttack>,
        ),
        (With<Player>, Without<InvincibilityTimer>, Without<MeleeAttack>),
    >,
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
                melee_attack,
                _hostile_transform,
                mut hostile_state,
                hostile_hit_timer,
                hostile_melee_timer,
                _level_entity_type,
            )) = enemy.get_mut(colliding_entity)
            {
                // Apply contact damage immediately (stomping removed) only if this hostile defines a melee attack.
                let dmg = melee_attack.damage;
                player_health.take_damage(dmg);
                // Mark entity for floating health text (negative = damage)
                commands
                    .entity(player_entity)
                    .insert(health_floating::RecentHealthChange(-(dmg)));
                commands
                    .entity(player_entity)
                    .insert(InvincibilityTimer::new(PLAYER_INVINCIBILITY_SECONDS));

                if !player_health.is_dead() {
                    player_state.set(EntityState::Hit);
                    commands
                        .entity(player_entity)
                        .insert(HitStateTimer::new(HIT_STATE_SECONDS, player_state.version));
                }

                if can_set_state(
                    &hostile_state,
                    hostile_hit_timer,
                    None,
                    hostile_melee_timer,
                    EntityState::MeleeAttack,
                ) {
                    hostile_state.set(EntityState::MeleeAttack);
                    commands
                        .entity(hostile_entity)
                        .insert(MeleeAttackStateTimer::new(MELEE_ATTACK_STATE_SECONDS));
                }

                info!(
                    "Player took {} damage from hostile - HP: {}/{}",
                    dmg, player_health.current, player_health.max
                );
                break;
            }
        }
    }
}
