use bevy::prelude::*;

use crate::game::components::plasma::{
    PLASMA_IMPACT_LIFETIME_SECS, PLASMA_IMPACT_MAX_SPEED, PLASMA_IMPACT_MIN_SPEED,
    PLASMA_IMPACT_PARTICLE_COUNT, PLASMA_Z,
};
use crate::game::gfx::helpers::PlasmaImpactParticle;
use crate::game::runtime_components::GameEntity;

use super::helpers::hash_to_unit;

pub(crate) fn spawn_plasma_impact_explosion(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    impact_position: Vec2,
) {
    for index in 0..PLASMA_IMPACT_PARTICLE_COUNT {
        let seed = index as u32 + 101;
        let angle = hash_to_unit(seed.wrapping_mul(37)) * std::f32::consts::TAU;
        let speed = PLASMA_IMPACT_MIN_SPEED
            + hash_to_unit(seed.wrapping_mul(71))
                * (PLASMA_IMPACT_MAX_SPEED - PLASMA_IMPACT_MIN_SPEED);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        let size = 4.0 + hash_to_unit(seed.wrapping_mul(13)) * 8.0;

        commands.spawn((
            Name::new("PlasmaImpactParticle"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.45, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, PLASMA_Z + 0.5),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS, TimerMode::Once),
                start_size: size,
            },
            GameEntity,
        ));
    }

    commands.spawn((
        Name::new("PlasmaImpactFlash"),
        Sprite {
            image: particle_image.clone(),
            color: Color::srgba(0.65, 1.0, 1.0, 0.75),
            custom_size: Some(Vec2::splat(46.0)),
            ..default()
        },
        Transform::from_xyz(impact_position.x, impact_position.y, PLASMA_Z + 0.6),
        PlasmaImpactParticle {
            velocity: Vec2::ZERO,
            lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS * 0.55, TimerMode::Once),
            start_size: 46.0,
        },
        GameEntity,
    ));
}
