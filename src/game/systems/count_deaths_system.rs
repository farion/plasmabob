use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::components::Health;
use crate::game::tags::EnemyTag;

/// Counts enemy deaths once per entity and keeps score in sync.
pub fn count_deaths_system(
	mut stats: ResMut<crate::LevelStats>,
	mut counted: Local<HashSet<Entity>>,
	enemies: Query<(Entity, &Health), With<EnemyTag>>,
) {
	for (entity, health) in &enemies {
		if health.is_dead() {
			if counted.insert(entity) {
				stats.enemies_killed = stats.enemies_killed.saturating_add(1);
				stats.recompute_score();
			}
		} else {
			counted.remove(&entity);
		}
	}

	// Drop entities that no longer exist in the query to avoid stale local state.
	counted.retain(|entity| enemies.get(*entity).is_ok());
}

