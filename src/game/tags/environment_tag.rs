use bevy::prelude::Component;

/// Marker component for static environment entities (ground, platforms, walls).
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EnvironmentTag;

