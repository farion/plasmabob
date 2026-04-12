use bevy::prelude::Component;

/// Marker component attached to every entity that is spawned as part of a
/// level. Enables bulk despawn of all level entities on GameView exit without
/// touching persistent entities such as the camera or UI.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GameEntity;

