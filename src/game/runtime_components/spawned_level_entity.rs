use bevy::prelude::Component;

/// Carries the original level JSON id and entity_type for spawned level
/// entities so runtime systems (eg. debug overlays) can display them.
#[derive(Component, Debug, Clone)]
pub struct SpawnedLevelEntity {
    pub id: String,
    pub entity_type: String,
    pub layer: String,
}

