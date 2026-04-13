use bevy::prelude::*;

use crate::game::components::plasma::{
    PLASMA_BEAM_PARTICLE_COUNT, PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE,
    PLASMA_BEAM_PARTICLE_WIGGLE_SPEED, PLASMA_BEAM_VISUAL_HALF_HEIGHT,
};
use crate::game::gfx::helpers::PlasmaBeamParticle;
use crate::helper::particles::create_round_particle_image;

use super::helpers::hash_to_unit;

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

pub(crate) fn spawn_plasma_beam_particles(
    beam_entity: &mut EntityCommands,
    particle_image: &Handle<Image>,
) {
    beam_entity.with_children(|parent| {
        for index in 0..PLASMA_BEAM_PARTICLE_COUNT {
            let seed = index as u32 + 1;
            let normalized_distance = if PLASMA_BEAM_PARTICLE_COUNT <= 1 {
                1.0
            } else {
                index as f32 / (PLASMA_BEAM_PARTICLE_COUNT - 1) as f32
            };
            let lane = ((hash_to_unit(seed.wrapping_mul(29)) * 2.0) - 1.0).powi(3);
            let phase = hash_to_unit(seed.wrapping_mul(53)) * std::f32::consts::TAU;
            let core_size = 4.0 + hash_to_unit(seed.wrapping_mul(97)) * 5.0;
            let glow_size = core_size * 2.0;
            let alpha = 0.55 + hash_to_unit(seed.wrapping_mul(11)) * 0.25;

            parent.spawn((
                Sprite {
                    color: Color::srgba(0.2, 0.98, 1.0, alpha),
                    custom_size: Some(Vec2::splat(core_size)),
                    ..Sprite::from_image(particle_image.clone())
                },
                Transform::from_xyz(0.0, 0.0, hash_to_unit(seed.wrapping_mul(7)) * 0.2),
                PlasmaBeamParticle {
                    normalized_distance,
                    lane,
                    phase,
                    layer_scale: 1.0,
                },
            ));

            parent.spawn((
                Sprite {
                    color: Color::srgba(0.12, 0.75, 1.0, alpha * 0.45),
                    custom_size: Some(Vec2::splat(glow_size)),
                    ..Sprite::from_image(particle_image.clone())
                },
                Transform::from_xyz(0.0, 0.0, -0.1 + hash_to_unit(seed.wrapping_mul(17)) * 0.15),
                PlasmaBeamParticle {
                    normalized_distance,
                    lane,
                    phase: phase + 0.9,
                    layer_scale: 1.8,
                },
            ));
        }
    });
}

pub(crate) fn update_beam_particles<
    F: bevy::ecs::query::QueryFilter,
    I: IntoIterator<Item = Entity>,
>(
    time: &Time,
    beam_direction: f32,
    beam_length: f32,
    children: I,
    beam_particles: &mut Query<(&PlasmaBeamParticle, &mut Transform, &mut Sprite), F>,
    alpha_multiplier: f32,
) {
    for child in children.into_iter() {
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
            beam_direction * beam_length * particle.normalized_distance;
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
