use bevy::prelude::*;

use crate::game::systems::gameplay::types::RangeProjectile;

pub(crate) fn move_range_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut RangeProjectile, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (projectile_entity, mut projectile, mut transform) in &mut projectiles {
        // store previous position for this tick
        let previous_position = transform.translation.truncate();
        projectile.previous_position = previous_position;

        // move projectile by its velocity
        transform.translation += (projectile.velocity * delta).extend(0.0);

        let current_position = transform.translation.truncate();

        // accumulate traveled distance this frame; use the movement delta so the check
        // remains accurate even if the projectile is moved externally.
        let frame_travel = current_position.distance(previous_position);
        projectile.traveled += frame_travel;

        // Despawn if accumulated travel exceeds max_range. Keep the original start_position
        // direct distance check as a conservative fallback.
        if projectile.traveled >= projectile.max_range
            || projectile.start_position.distance(current_position) >= projectile.max_range
        {
            commands.entity(projectile_entity).despawn();
        }
    }
}

