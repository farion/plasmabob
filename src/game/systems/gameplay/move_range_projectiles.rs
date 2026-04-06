use bevy::prelude::*;

use crate::game::systems::gameplay::types::RangeProjectile;

pub(crate) fn move_range_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut RangeProjectile, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (projectile_entity, mut projectile, mut transform) in &mut projectiles {
        let previous_position = transform.translation.truncate();
        projectile.previous_position = previous_position;

        transform.translation += (projectile.velocity * delta).extend(0.0);

        let current_position = transform.translation.truncate();
        if projectile.start_position.distance(current_position) >= projectile.max_range {
            commands.entity(projectile_entity).despawn();
        }
    }
}

