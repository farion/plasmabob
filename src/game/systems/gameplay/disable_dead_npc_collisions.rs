use bevy::prelude::*;
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, LinearVelocity, LockedAxes, RigidBody};

use crate::game::components::npc::Npc;
use crate::game::components::collision::Collision;
use crate::game::components::health::Health;
use crate::game::systems::gameplay::types::DeadNpcCollisionDisabled;

pub(crate) fn disable_dead_npc_collisions(
    mut commands: Commands,
    dead_npcs: Query<(Entity, &Health), (With<Npc>, With<Collision>, Without<DeadNpcCollisionDisabled>)>,
) {
    for (entity, health) in &dead_npcs {
        if !health.is_dead() {
            continue;
        }

        commands.entity(entity).remove::<(
            Collision,
            Collider,
            CollidingEntities,
            CollisionLayers,
            RigidBody,
            LinearVelocity,
            LockedAxes,
        )>();
        commands.entity(entity).insert(DeadNpcCollisionDisabled);
    }
}


