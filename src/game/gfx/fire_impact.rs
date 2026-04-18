use bevy::prelude::*;

use crate::game::gfx::helpers::{hash_to_unit, PlasmaImpactParticle};
use crate::game::runtime_components::GameEntity;

/// Spawn a small fire explosion at the given impact position.
///
/// Uses three layers of particles:
/// - **Fire burst** – hot orange/yellow blobs flying outward with an upward bias (fire rises).
/// - **Ember sparks** – tiny bright specks shooting out at high speed, short-lived.
/// - **Smoke puffs** – large, dark, slow-rising wisps that linger longest.
pub(crate) fn spawn_fire_impact_explosion(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    impact_position: Vec2,
    impact_z: f32,
) {
    let z = impact_z + 0.25;
    let base_seed = impact_position.x.to_bits().wrapping_mul(31)
        ^ impact_position.y.to_bits().wrapping_mul(131)
        ^ 0xF1A3_E5B7;

    // Fire burst: more and stronger orange/red/yellow blobs flying outward, with upward bias.
    for i in 0..24 {
        let seed = base_seed.wrapping_add((i as u32).wrapping_mul(1_301));
        let angle = hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU;
        // stronger, wider speed range for a beefier explosion
        let speed = 150.0 + hash_to_unit(seed.wrapping_mul(11)) * 260.0;
        let upward_bias = 60.0 + hash_to_unit(seed.wrapping_mul(12)) * 90.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed + upward_bias);
        // larger blobs
        let size = 14.0 + hash_to_unit(seed.wrapping_mul(13)) * 20.0;
        // slightly longer-lived
        let ttl = 0.22 + hash_to_unit(seed.wrapping_mul(17)) * 0.30;
        let heat = hash_to_unit(seed.wrapping_mul(19));

        commands.spawn((
            Name::new("FireImpactBurst"),
            Sprite {
                image: particle_image.clone(),
                // Range from deep orange to bright yellow (keep saturated)
                color: Color::srgba(1.0, 0.30 + heat * 0.68, heat * 0.12, 1.0),
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

    // Ember sparks: more numerous, brighter specks at higher velocity.
    for i in 0..22 {
        let seed = base_seed
            .wrapping_add(0x9E37_79B9)
            .wrapping_add((i as u32).wrapping_mul(1_777));
        let angle = hash_to_unit(seed.wrapping_mul(23)) * std::f32::consts::TAU;
        // increased speed for more dramatic embers
        let speed = 320.0 + hash_to_unit(seed.wrapping_mul(29)) * 260.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        // slightly larger embers
        let size = 5.0 + hash_to_unit(seed.wrapping_mul(37)) * 6.0;
        let ttl = 0.22 + hash_to_unit(seed.wrapping_mul(41)) * 0.30;
        let brightness = 0.80 + hash_to_unit(seed.wrapping_mul(43)) * 0.25;

        commands.spawn((
            Name::new("FireImpactEmber"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(1.0, brightness, 0.18, 1.0),
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

    // Smoke puffs: larger, denser, slow-rising blobs that linger longer.
    for i in 0..14 {
        let seed = base_seed
            .wrapping_add(0x4CF5_AD43)
            .wrapping_add((i as u32).wrapping_mul(2_441));
        let angle = hash_to_unit(seed.wrapping_mul(43)) * std::f32::consts::TAU;
        let speed = 45.0 + hash_to_unit(seed.wrapping_mul(47)) * 90.0;
        // Smoke always drifts upward regardless of the lateral angle
        let velocity = Vec2::new(
            angle.cos() * speed,
            angle.sin().abs() * speed + 60.0 + hash_to_unit(seed.wrapping_mul(53)) * 70.0,
        );
        // bigger, bulkier smoke
        let size = 26.0 + hash_to_unit(seed.wrapping_mul(59)) * 26.0;
        // last noticeably longer
        let ttl = 0.48 + hash_to_unit(seed.wrapping_mul(61)) * 0.36;
        let dark = hash_to_unit(seed.wrapping_mul(67));

        commands.spawn((
            Name::new("FireImpactSmoke"),
            Sprite {
                image: particle_image.clone(),
                // Charcoal-brown to ashy grey, slightly more opaque
                color: Color::srgba(0.32 + dark * 0.22, 0.08 + dark * 0.10, 0.02, 0.75),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, z - 0.02),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(ttl, TimerMode::Once),
                start_size: size,
            },
            GameEntity,
        ));
    }
}

