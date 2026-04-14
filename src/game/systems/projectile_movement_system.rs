use bevy::prelude::*;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::RigidBody;
use crate::game::runtime_components::Projectile;

const BEAM_AFTERGLOW_SECS: f32 = 0.28;

pub fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &RigidBody, &mut Projectile)>,
    mut beams: Query<(Entity, &mut PlasmaBeam)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let mut expired_projectiles: Vec<(Entity, Vec2, f32)> = Vec::new();

    for (projectile_entity, mut transform, rigid_body, mut projectile) in &mut projectiles {
        let motion = rigid_body.velocity * dt;
        transform.translation.x += motion.x;
        transform.translation.y += motion.y;
        projectile.remaining_range = (projectile.remaining_range - motion.length()).max(0.0);

        if projectile.remaining_range <= f32::EPSILON {
            let impact_position = transform.translation.truncate();
            let impact_z = transform.translation.z;
            // Do NOT spawn impact or sfx here — impact should only be created when a projectile
            // actually hits an entity (handled in projectile_collision_system). When the
            // projectile simply expires due to range, we only set beams to afterglow and despawn.
            expired_projectiles.push((projectile_entity, impact_position, impact_z));
        }
    }

    if expired_projectiles.is_empty() {
        return;
    }

    for (projectile_entity, _impact_position, _impact_z) in expired_projectiles {
        // Only set the beam to afterglow and despawn the projectile. Do not spawn impact
        // visuals or sounds for range expiration — impacts are only for real hits.
        set_beams_to_afterglow(projectile_entity, &mut beams);
        commands.entity(projectile_entity).despawn();
    }
}

fn set_beams_to_afterglow(projectile_entity: Entity, beams: &mut Query<(Entity, &mut PlasmaBeam)>) {
    for (_beam_entity, mut beam) in beams {
        if beam.target_projectile == Some(projectile_entity) {
            beam.target_projectile = None;
            beam.lifetime = Some(Timer::from_seconds(BEAM_AFTERGLOW_SECS, TimerMode::Once));
        }
    }
}

