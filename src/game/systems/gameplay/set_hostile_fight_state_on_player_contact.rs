use std::collections::HashSet;
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer, can_set_state, MeleeAttackStateTimer, MELEE_ATTACK_STATE_SECONDS};
use crate::game::components::hostile::Hostile;
use crate::game::components::player::Player;

pub(crate) fn set_hostile_fight_state_on_player_contact(
    mut commands: Commands,
    player_entities: Query<Entity, With<Player>>,
    mut hostiles: Query<(
        Entity,
        &CollidingEntities,
        &mut AnimationState,
        Option<&HitStateTimer>,
        Option<&MeleeAttackStateTimer>,
        Option<&crate::game::components::health::Health>,
    ), (With<Hostile>, Without<Player>)>,
) {
    let player_set: HashSet<Entity> = player_entities.iter().collect();

    for (hostile_entity, colliding_entities, mut hostile_state, hit_timer, fight_timer, health) in &mut hostiles {
        if health.is_some_and(|value| value.is_dead()) {
            continue;
        }

        let touches_player = colliding_entities
            .0
            .iter()
            .any(|entity| player_set.contains(entity));

        if !touches_player {
            continue;
        }

            if can_set_state(&hostile_state, hit_timer, None, fight_timer, EntityState::MeleeAttack) {
                hostile_state.set(EntityState::MeleeAttack);
                commands
                    .entity(hostile_entity)
                    .insert(MeleeAttackStateTimer::new(MELEE_ATTACK_STATE_SECONDS));
            }
    }
}

