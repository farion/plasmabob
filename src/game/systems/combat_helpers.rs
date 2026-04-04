use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::plasma::{
    PLASMA_Z,
    PLASMA_IMPACT_PARTICLE_COUNT,
    PLASMA_IMPACT_MAX_SPEED,
    PLASMA_IMPACT_MIN_SPEED,
    PLASMA_IMPACT_LIFETIME_SECS,
    PLASMA_BEAM_PARTICLE_WIGGLE_SPEED,
    PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE,
    PLASMA_BEAM_VISUAL_HALF_HEIGHT,
};

use crate::game::systems::combat_types::PlasmaBeamParticle;
use crate::game::view_api::GameViewEntity;

/// Return a deterministic pseudo-random value in [0,1) from a seed.
pub(crate) fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}

pub(crate) fn create_round_particle_image(size: u32) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center.max(1.0);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt() / radius;
            let softness = (1.0 - distance).clamp(0.0, 1.0);
            let alpha = (softness * softness * 255.0) as u8;

            let index = ((y * size + x) * 4) as usize;
            data[index] = 255;
            data[index + 1] = 255;
            data[index + 2] = 255;
            data[index + 3] = alpha;
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(crate) fn ensure_plasma_particle_image(local_handle: &mut Option<Handle<Image>>, images: &mut Assets<Image>) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }
    let handle = images.add(create_round_particle_image(32));
    *local_handle = Some(handle.clone());
    handle
}

pub(crate) fn plasma_origin_from_player(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    let y_from_bottom = size.y * crate::game::components::plasma::PLASMA_ORIGIN_HEIGHT_RATIO_FROM_BOTTOM;
    let y = transform.translation.y - (size.y * 0.5) + y_from_bottom;
    Vec2::new(transform.translation.x, y)
}

pub(crate) fn update_beam_particles<F: bevy::ecs::query::QueryFilter>(
    time: &Time,
    beam: &PlasmaBeam,
    children: &Children,
    beam_particles: &mut Query<(&PlasmaBeamParticle, &mut Transform, &mut Sprite), F>,
    alpha_multiplier: f32,
) {
    for child in children.iter() {
        let Ok((particle, mut particle_transform, mut particle_sprite)) = beam_particles.get_mut(child)
        else { continue; };

        let wave = (time.elapsed_secs() * PLASMA_BEAM_PARTICLE_WIGGLE_SPEED + particle.phase).sin();
        let taper = 1.0 - (particle.normalized_distance * 0.45);
        let y_offset = (particle.lane * PLASMA_BEAM_VISUAL_HALF_HEIGHT)
            + (wave * PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE * taper * particle.layer_scale);

        particle_transform.translation.x = beam.direction * beam.current_length * particle.normalized_distance;
        particle_transform.translation.y = y_offset;

        let core_boost = 1.0 - particle.lane.abs() * 0.45;
        let alpha = (0.35 + core_boost * 0.65) * alpha_multiplier;
        let color = if particle.layer_scale > 1.0 {
            Color::srgba(0.1, 0.75, 1.0, (alpha * 0.5).clamp(0.0, 1.0))
        } else {
            Color::srgba(0.25, 1.0, 1.0, alpha.clamp(0.0, 1.0))
        };
        particle_sprite.color = color;
    }
}

pub(crate) fn spawn_plasma_impact_explosion(commands: &mut Commands, particle_image: &Handle<Image>, impact_position: Vec2) {
    for index in 0..PLASMA_IMPACT_PARTICLE_COUNT {
        let seed = index as u32 + 101;
        let angle = hash_to_unit(seed.wrapping_mul(37)) * std::f32::consts::TAU;
        let speed = PLASMA_IMPACT_MIN_SPEED
            + hash_to_unit(seed.wrapping_mul(71)) * (PLASMA_IMPACT_MAX_SPEED - PLASMA_IMPACT_MIN_SPEED);
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
            crate::game::systems::combat_types::PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS, TimerMode::Once),
                start_size: size,
            },
            GameViewEntity,
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
        crate::game::systems::combat_types::PlasmaImpactParticle {
            velocity: Vec2::ZERO,
            lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS * 0.55, TimerMode::Once),
            start_size: 46.0,
        },
        GameViewEntity,
    ));
}

