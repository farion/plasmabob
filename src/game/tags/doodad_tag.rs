use bevy::prelude::Component;

/// Marker component for decorative / non-interactive entities (flora, mountains, …).
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DoodadTag;

