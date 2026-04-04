use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct DustParticle {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: Timer,
    pub(crate) start_size: f32,
}

