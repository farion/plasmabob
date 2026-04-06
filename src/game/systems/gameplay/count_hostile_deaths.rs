use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::components::hostile::Hostile;
use crate::game::systems::gameplay::types::DeathCounted;

pub(crate) fn count_hostile_deaths(
    mut commands: Commands,
    mut stats: ResMut<crate::LevelStats>,
    dead_hostiles: Query<
        (Entity, &crate::game::components::health::Health),
        (
            With<Hostile>,
            With<SpawnedLevelEntity>,
            Without<DeathCounted>,
        ),
    >,
) {
    for (entity, health) in &dead_hostiles {
        if !health.is_dead() {
            continue;
        }
        stats.enemies_killed = stats.enemies_killed.saturating_add(1);
        commands.entity(entity).insert(DeathCounted);
    }
}
