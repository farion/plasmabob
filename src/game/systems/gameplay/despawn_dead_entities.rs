use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::components::health::Health;
use crate::game::components::npc::Npc;
use crate::game::components::player::Player;

pub(crate) fn despawn_dead_entities(
    mut commands: Commands,
    dead_query: Query<(Entity, &Health), (Without<Player>, Without<Npc>, With<SpawnedLevelEntity>)>,
) {
    for (entity, health) in &dead_query {
        if health.is_dead() {
            info!("Entity {:?} died - despawning.", entity);
            commands.entity(entity).despawn();
        }
    }
}
