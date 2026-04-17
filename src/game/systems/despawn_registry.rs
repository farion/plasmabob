use bevy::prelude::*;
use std::collections::HashSet;

/// Resource tracking entities that have been scheduled for despawn this frame.
///
/// This avoids queuing duplicate despawn commands for the same entity when multiple
/// systems (collision, movement expiry, etc.) may attempt to despawn the same projectile
/// within the same update tick. Commands are deferred in Bevy which means the entity
/// is still present during the frame and naive despawn calls can be queued multiple
/// times resulting in warnings when the deferred queue is applied.
#[derive(Resource, Default)]
pub struct DespawnRegistry {
    set: HashSet<Entity>,
}

impl DespawnRegistry {
    pub fn contains(&self, e: Entity) -> bool {
        self.set.contains(&e)
    }

    pub fn insert(&mut self, e: Entity) {
        self.set.insert(e);
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }
}

/// Clear the registry at the end of the frame so subsequent frames start fresh.
pub fn clear_registry_system(mut registry: ResMut<DespawnRegistry>) {
    registry.clear();
}


