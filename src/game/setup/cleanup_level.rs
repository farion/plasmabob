use bevy::prelude::*;

use crate::game::runtime_components::GameEntity;
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::game::systems::sound_system::StateSoundLoop;


/// Despawns every entity tagged with [`GameEntity`] when the GameView is exited.
/// Also stops any active state-sound loop entities and removes the level asset cache.
pub fn cleanup_game_entities(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    loop_sounds: Query<Entity, With<StateSoundLoop>>,
) {
    // Stop loop sounds before despawning game entities.
    let mut loop_count = 0u32;
    for entity in &loop_sounds {
        commands.entity(entity).try_despawn();
        loop_count += 1;
    }

    let mut count = 0u32;
    for entity in &game_entities {
        commands.entity(entity).try_despawn();
        count += 1;
    }

    // Free preloaded handles so Bevy's asset GC can release GPU memory.
    commands.remove_resource::<EntityTypeAssets>();

    tracing::info!(
        entities = count,
        loop_sounds = loop_count,
        "Cleaned up game entities and asset cache on GameView exit"
    );
}
