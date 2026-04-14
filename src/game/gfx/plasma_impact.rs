use bevy::prelude::*;

use crate::game::gfx::helpers::PlasmaImpactParticle;
use crate::game::gfx::plasma_sizes::{
    PLASMA_IMPACT_AFTERGLOW_SIZE_MIN, PLASMA_IMPACT_AFTERGLOW_SIZE_RANGE,
    PLASMA_IMPACT_ARC_SIZE_MIN, PLASMA_IMPACT_ARC_SIZE_RANGE, PLASMA_IMPACT_BURST_SIZE_MIN,
    PLASMA_IMPACT_BURST_SIZE_RANGE,
};
use crate::game::runtime_components::GameEntity;

use super::helpers::hash_to_unit;

pub(crate) fn spawn_plasma_impact_explosion(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    impact_position: Vec2,
    impact_z: f32,
) {
    let z = impact_z + 0.25;
    let base_seed = impact_position.x.to_bits().wrapping_mul(31)
        ^ impact_position.y.to_bits().wrapping_mul(131)
        ^ 0xA531_77D3;

    for i in 0..18 {
        let seed = base_seed.wrapping_add((i as u32).wrapping_mul(1_301));
        let angle = hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU;
        let speed = 220.0 + hash_to_unit(seed.wrapping_mul(11)) * 310.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        let size =
            PLASMA_IMPACT_BURST_SIZE_MIN + hash_to_unit(seed.wrapping_mul(13)) * PLASMA_IMPACT_BURST_SIZE_RANGE;
        let ttl = 0.22 + hash_to_unit(seed.wrapping_mul(17)) * 0.16;
        let violet = hash_to_unit(seed.wrapping_mul(19)) * 0.55;

        commands.spawn((
            Name::new("PlasmaImpactBurst"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.62 + violet * 0.33, 0.92 + (1.0 - violet) * 0.08, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, z),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(ttl, TimerMode::Once),
                start_size: size,
            },
            GameEntity,
        ));
    }

    for i in 0..22 {
        let seed = base_seed
            .wrapping_add(0x9E37_79B9)
            .wrapping_add((i as u32).wrapping_mul(1_777));
        let angle = hash_to_unit(seed.wrapping_mul(23)) * std::f32::consts::TAU;
        let speed = 80.0 + hash_to_unit(seed.wrapping_mul(29)) * 150.0;
        let radial = Vec2::new(angle.cos(), angle.sin());
        let tangential = Vec2::new(-radial.y, radial.x)
            * ((hash_to_unit(seed.wrapping_mul(31)) - 0.5) * 120.0);
        let velocity = radial * speed + tangential;
        let size = PLASMA_IMPACT_AFTERGLOW_SIZE_MIN
            + hash_to_unit(seed.wrapping_mul(37)) * PLASMA_IMPACT_AFTERGLOW_SIZE_RANGE;
        let ttl = 0.45 + hash_to_unit(seed.wrapping_mul(41)) * 0.25;

        commands.spawn((
            Name::new("PlasmaImpactAfterglow"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.42, 0.9, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, z - 0.01),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(ttl, TimerMode::Once),
                start_size: size,
            },
            GameEntity,
        ));
    }

    for i in 0..14 {
        let seed = base_seed
            .wrapping_add(0x4CF5_AD43)
            .wrapping_add((i as u32).wrapping_mul(2_441));
        let forward_sign = if hash_to_unit(seed.wrapping_mul(43)) > 0.5 {
            1.0
        } else {
            -1.0
        };
        let velocity = Vec2::new(
            forward_sign * (160.0 + hash_to_unit(seed.wrapping_mul(47)) * 200.0),
            (hash_to_unit(seed.wrapping_mul(53)) - 0.5) * 260.0,
        );
        let size =
            PLASMA_IMPACT_ARC_SIZE_MIN + hash_to_unit(seed.wrapping_mul(59)) * PLASMA_IMPACT_ARC_SIZE_RANGE;
        let ttl = 0.32 + hash_to_unit(seed.wrapping_mul(61)) * 0.18;

        commands.spawn((
            Name::new("PlasmaImpactArcSpark"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.98, 0.9, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, z + 0.02),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(ttl, TimerMode::Once),
                start_size: size,
            },
            GameEntity,
        ));
    }
}
