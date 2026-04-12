use bevy::prelude::*;

use crate::game::gameplay::components::GameEntity;

/// Despawns every entity tagged with [`GameEntity`] when the GameView is exited.
/// This clears the spawned level (sprites, entities) without affecting
/// persistent entities such as the camera or UI roots.
pub fn cleanup_game_entities(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
) {
    let mut count = 0u32;
    for entity in &game_entities {
        commands.entity(entity).despawn();
        count += 1;
    }
    tracing::info!(count, "Cleaned up game entities on GameView exit");
}

