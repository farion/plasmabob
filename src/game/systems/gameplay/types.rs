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

#[derive(Component)]
pub(crate) struct DustParticle {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: Timer,
    pub(crate) start_size: f32,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct RangeProjectile {
    pub(crate) shooter: Entity,
    pub(crate) start_position: Vec2,
    pub(crate) previous_position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) damage: i32,
    pub(crate) max_range: f32,
}

impl RangeProjectile {
    pub(crate) fn new(
        shooter: Entity,
        start_position: Vec2,
        velocity: Vec2,
        damage: i32,
        max_range: f32,
    ) -> Self {
        Self {
            shooter,
            start_position,
            previous_position: start_position,
            velocity,
            damage: damage.max(0),
            max_range: max_range.max(1.0),
        }
    }
}

