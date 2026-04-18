use bevy::prelude::Component;

/// Marker component: this entity blocks movement/collisions (e.g. walls, platforms).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Blocking;

