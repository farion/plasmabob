use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct DeathQuotePlayed;

#[derive(Component)]
pub(crate) struct DeathCounted;

#[derive(Component)]
pub(crate) struct PlasmaBeamParticle {
    pub(crate) normalized_distance: f32,
    pub(crate) lane: f32,
    pub(crate) phase: f32,
    pub(crate) layer_scale: f32,
}

#[derive(Component)]
pub(crate) struct PlasmaImpactParticle {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: Timer,
    pub(crate) start_size: f32,
}

#[derive(Component)]
pub(crate) struct DeadNpcCollisionDisabled;

