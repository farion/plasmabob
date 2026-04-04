use bevy::prelude::*;

pub(crate) fn detect_player_collectibles(
    mut commands: Commands,
    mut player_query: Query<(Entity, &avian2d::prelude::CollidingEntities, &mut crate::game::components::health::Health), With<crate::game::components::player::Player>>,
    collectible_query: Query<(), With<crate::game::components::collectible::Collectible>>,
    effect_heal_query: Query<&crate::game::components::effect_heal::EffectHeal>,
) {
    for (_player_entity, colliding_entities, mut player_health) in &mut player_query {
        if player_health.is_dead() { continue; }

        for &colliding_entity in colliding_entities.0.iter() {
            if collectible_query.get(colliding_entity).is_ok() {
                if let Ok(effect) = effect_heal_query.get(colliding_entity) {
                    let heal_amount = effect.0;
                    player_health.current = (player_health.current + heal_amount).min(player_health.max);
                    info!("Player healed by {} - HP: {}/{}", heal_amount, player_health.current, player_health.max);
                    commands.entity(_player_entity).insert(super::health_floating::RecentHealthChange(heal_amount));
                }
                commands.entity(colliding_entity).despawn();
            }
        }
    }
}

