use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::plasma::{
    PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE, PLASMA_BEAM_PARTICLE_WIGGLE_SPEED,
    PLASMA_BEAM_VISUAL_HALF_HEIGHT, PLASMA_IMPACT_LIFETIME_SECS, PLASMA_IMPACT_MAX_SPEED,
    PLASMA_IMPACT_MIN_SPEED, PLASMA_IMPACT_PARTICLE_COUNT, PLASMA_Z,
};
use crate::game::systems::gameplay::types::{
    DustParticle, PlasmaBeamParticle, PlasmaImpactParticle,
};
use crate::game::systems::systems_api::GameViewEntity;
use crate::helper::particles::create_round_particle_image;

/// Return a deterministic pseudo-random value in [0,1) from a seed.
pub(crate) fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}

pub(crate) fn ensure_plasma_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }
    let handle = images.add(create_round_particle_image(32));
    *local_handle = Some(handle.clone());
    handle
}

pub(crate) fn plasma_origin_from_player(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    let y_from_bottom =
        size.y * crate::game::components::plasma::PLASMA_ORIGIN_HEIGHT_RATIO_FROM_BOTTOM;
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
        let Ok((particle, mut particle_transform, mut particle_sprite)) =
            beam_particles.get_mut(child)
        else {
            continue;
        };

        let wave = (time.elapsed_secs() * PLASMA_BEAM_PARTICLE_WIGGLE_SPEED + particle.phase).sin();
        let taper = 1.0 - (particle.normalized_distance * 0.45);
        let y_offset = (particle.lane * PLASMA_BEAM_VISUAL_HALF_HEIGHT)
            + (wave * PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE * taper * particle.layer_scale);

        particle_transform.translation.x =
            beam.direction * beam.current_length * particle.normalized_distance;
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
        PlasmaImpactParticle {
            velocity: Vec2::ZERO,
            lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS * 0.55, TimerMode::Once),
            start_size: 46.0,
        },
        GameViewEntity,
    ));
}

pub(crate) fn dust_origin(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    Vec2::new(
        transform.translation.x,
        transform.translation.y - (size.y * 0.45),
    )
}

pub(crate) fn spawn_dust_burst(
    commands: &mut Commands,
    origin: Vec2,
    particle_image: &Handle<Image>,
    count: usize,
    seed_offset: u32,
    upward_speed: f32,
) {
    for index in 0..count {
        let seed = seed_offset.wrapping_add(index as u32 + 1);
        let spread = (hash_to_unit(seed.wrapping_mul(13)) * 2.0) - 1.0;
        let horizontal = spread * 170.0;
        let upward = upward_speed * (0.45 + hash_to_unit(seed.wrapping_mul(29)) * 0.75);
        let size = 6.0 + hash_to_unit(seed.wrapping_mul(47)) * 8.0;

        commands.spawn((
            Name::new("DustParticle"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.55, 0.55, 0.55, 0.7),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(origin.x, origin.y, 9.0),
            DustParticle {
                velocity: Vec2::new(horizontal, upward),
                lifetime: Timer::from_seconds(0.28, TimerMode::Once),
                start_size: size,
            },
            GameViewEntity,
        ));
    }
}

pub(crate) fn ensure_dust_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }

    let handle = images.add(create_round_particle_image(24));
    *local_handle = Some(handle.clone());
    handle
}

pub(crate) fn update_sprite_flip_for_move_axis(sprite: &mut Sprite, move_axis: f32) {
    if move_axis < 0.0 && !sprite.flip_x {
        sprite.flip_x = true;
    } else if move_axis > 0.0 && sprite.flip_x {
        sprite.flip_x = false;
    }
}

/// If a small vertical step is detected ahead of the entity, nudge the entity up
/// so it can continue moving instead of getting stuck. The tolerance parameter
/// is the maximum vertical step (in pixels) we will climb.
pub(crate) fn detect_small_step(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &Transform,
    _sprite: &Sprite,
    direction: f32,
    max_step: f32,
) -> Option<f32> {
    let foot_offset = -10.0;
    let probe_x = transform.translation.x + (direction * 8.0);
    let probe_y = transform.translation.y + foot_offset;

    let origin_current = Vec2::new(transform.translation.x, probe_y);
    let origin_ahead = Vec2::new(probe_x, probe_y);

    let mut filter = SpatialQueryFilter::default();
    filter.excluded_entities.insert(entity);

    let hits_current = spatial_query.ray_hits(origin_current, Dir2::NEG_Y, 40.0, 8, true, &filter);
    let hits_ahead = spatial_query.ray_hits(origin_ahead, Dir2::NEG_Y, 40.0, 8, true, &filter);

    if hits_current.is_empty() || hits_ahead.is_empty() {
        return None;
    }

    let current_min = hits_current
        .iter()
        .map(|h| h.distance)
        .fold(f32::INFINITY, f32::min);
    let ahead_min = hits_ahead
        .iter()
        .map(|h| h.distance)
        .fold(f32::INFINITY, f32::min);

    let current_ground_y = origin_current.y - current_min;
    let ahead_ground_y = origin_ahead.y - ahead_min;

    if ahead_ground_y > current_ground_y {
        let step = ahead_ground_y - current_ground_y;
        if step > 0.5 && step <= max_step {
            // Return a small upward velocity to climb the step; scale to avoid high jumps.
            return Some((step + 8.0).min(220.0));
        }
    }

    None
}
