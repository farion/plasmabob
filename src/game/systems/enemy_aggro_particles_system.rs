use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::components::{AutoMovement, AutoMovementState, Collider, StateMachine};
use crate::game::gfx::aggro::{spawn_aggro_sparks, AggroParticleImage};
use crate::game::tags::EnemyTag;

pub fn enemy_aggro_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    particle_image: Option<Res<AggroParticleImage>>,
    mut cooldowns: Local<HashMap<Entity, f32>>,
    enemies: Query<
        (
            Entity,
            &Transform,
            Option<&Collider>,
            &AutoMovement,
            Option<&StateMachine>,
        ),
        With<EnemyTag>,
    >,
) {
    let Some(particle_image) = particle_image else {
        return;
    };

    let dt = time.delta_secs();
    let now_seed = time.elapsed_secs_wrapped().to_bits();

    for (entity, transform, collider, auto, sm) in &enemies {
        if sm.is_some_and(|state_machine| state_machine.is_non_interactive()) {
            cooldowns.remove(&entity);
            continue;
        }

        if auto.state != AutoMovementState::Aggro || !auto.enabled {
            cooldowns.remove(&entity);
            continue;
        }

        let cooldown = cooldowns.entry(entity).or_insert(0.0);
        *cooldown -= dt;
        if *cooldown > 0.0 {
            continue;
        }

        let center = transform.translation.truncate()
            + collider.map(|shape| shape.offset).unwrap_or(Vec2::ZERO);
        let seed = entity
            .to_bits() as u32
            ^ now_seed
            ^ transform.translation.x.to_bits().wrapping_mul(31)
            ^ transform.translation.y.to_bits().wrapping_mul(131);

        spawn_aggro_sparks(
            &mut commands,
            &particle_image.0,
            center,
            transform.translation.z + 0.35,
            seed,
        );

        // Slight per-entity variance avoids all enemies pulsing in sync.
        let variance = ((entity.to_bits() as u32).wrapping_mul(7_919) % 17) as f32 * 0.002;
        *cooldown = 0.09 + variance;
    }

    cooldowns.retain(|entity, _| enemies.get(*entity).is_ok());
}

