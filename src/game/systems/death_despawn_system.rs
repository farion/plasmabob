use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::components::Health;

const DEATH_FADE_MS: f32 = 500.0;

/// Handles delayed despawn for dead entities based on [`Health`] config.
///
/// Behavior:
/// - `despawn_on_death = false` => never despawn automatically.
/// - `despawn_on_death = true` => wait `despawn_delay_ms`, then fade out for 500ms.
/// - After fade-out completes, the entity is despawned.
pub fn death_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut elapsed_by_entity_ms: Local<HashMap<Entity, f32>>,
    mut entities: Query<(Entity, &Health, Option<&mut Sprite>)>,
) {
    let dt_ms = (time.delta_secs() * 1000.0).max(0.0);

    for (entity, health, sprite) in &mut entities {
        if !health.is_dead() {
            elapsed_by_entity_ms.remove(&entity);
            continue;
        }

        if !health.despawn_on_death {
            elapsed_by_entity_ms.remove(&entity);
            continue;
        }

        let elapsed = elapsed_by_entity_ms.entry(entity).or_insert(0.0);
        *elapsed += dt_ms;

        let fade_start_ms = health.despawn_delay_ms as f32;
        if *elapsed >= fade_start_ms {
            if let Some(mut sprite) = sprite {
                let fade_fraction = ((*elapsed - fade_start_ms) / DEATH_FADE_MS).clamp(0.0, 1.0);
                sprite.color.set_alpha(1.0 - fade_fraction);
            }
        }

        if *elapsed >= fade_start_ms + DEATH_FADE_MS {
            elapsed_by_entity_ms.remove(&entity);
            // Use try_despawn to avoid warnings if another system already queued a despawn
            // for this entity earlier in the same frame.
            commands.entity(entity).try_despawn();
        }
    }

    // Remove stale local entries for entities that no longer match the query.
    elapsed_by_entity_ms.retain(|entity, _| entities.get(*entity).is_ok());
}
