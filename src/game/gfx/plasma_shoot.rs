use bevy::prelude::*;

use crate::game::gfx::helpers::PlasmaBeamParticle;
use crate::game::gfx::plasma_sizes::{
    PLASMA_ARC_AMPLITUDE, PLASMA_ARC_PARTICLE_SIZE, PLASMA_CORE_PARTICLE_SIZE,
    PLASMA_CORE_WIDTH, PLASMA_GLOW_PARTICLE_SIZE, PLASMA_GLOW_WIDTH,
    PLASMA_PARTICLE_TEXTURE_SIZE,
};
use crate::game::gfx::particles::create_plasma_particle_image;

use super::helpers::hash_to_unit;

const CORE_PARTICLES: usize = 18;
const GLOW_PARTICLES: usize = 20;
const ARC_PARTICLES: usize = 14;

const ARC_SPEED: f32 = 26.0;

pub(crate) fn ensure_plasma_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }

    let handle = images.add(create_plasma_particle_image(PLASMA_PARTICLE_TEXTURE_SIZE));
    *local_handle = Some(handle.clone());
    handle
}

#[derive(Resource)]
pub(crate) struct PlasmaParticleImage(pub Handle<Image>);

pub(crate) fn preload_plasma_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(create_plasma_particle_image(PLASMA_PARTICLE_TEXTURE_SIZE));
    commands.insert_resource(PlasmaParticleImage(handle));
}

/// Clean up the preloaded plasma particle image resource and remove the
/// generated Image from the asset storage. This should be called when the
/// level (GameView) exits so we don't keep the generated texture resident
/// across levels or in the main menu.
pub(crate) fn cleanup_plasma_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    plasma_res: Option<Res<PlasmaParticleImage>>,
) {
    if let Some(res) = plasma_res {
        // Remove the image asset from the Assets<Image> pool. Clone the
        // handle because `remove` takes ownership.
        let handle = res.0.clone();
        // `Assets::remove` expects the handle id/asset id; use `handle.id()`.
        images.remove(handle.id());
        // Remove the resource so subsequent levels can preload afresh.
        commands.remove_resource::<PlasmaParticleImage>();
    }
}

pub(crate) fn spawn_plasma_beam_particles(
    beam_entity: &mut EntityCommands,
    particle_image: &Handle<Image>,
) {
    beam_entity.with_children(|parent| {
        for i in 0..CORE_PARTICLES {
            let seed = (i as u32).wrapping_mul(11_123).wrapping_add(17);
            parent.spawn((
                Name::new("PlasmaBeamParticleCore"),
                Sprite {
                    image: particle_image.clone(),
                    color: Color::srgba(0.88, 1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::splat(PLASMA_CORE_PARTICLE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.12),
                PlasmaBeamParticle {
                    normalized_distance: hash_to_unit(seed.wrapping_mul(3)),
                    lane: (hash_to_unit(seed.wrapping_mul(5)) - 0.5) * 0.22,
                    phase: hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU,
                    layer_scale: 0.95 + hash_to_unit(seed.wrapping_mul(13)) * 0.25,
                },
            ));
        }

        for i in 0..GLOW_PARTICLES {
            let seed = (i as u32).wrapping_mul(19_493).wrapping_add(31);
            parent.spawn((
                Name::new("PlasmaBeamParticleGlow"),
                Sprite {
                    image: particle_image.clone(),
                    color: Color::srgba(0.48, 0.92, 1.0, 0.88),
                    custom_size: Some(Vec2::splat(PLASMA_GLOW_PARTICLE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
                PlasmaBeamParticle {
                    normalized_distance: hash_to_unit(seed.wrapping_mul(3)),
                    lane: (hash_to_unit(seed.wrapping_mul(5)) - 0.5) * 0.52,
                    phase: hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU,
                    layer_scale: 1.15 + hash_to_unit(seed.wrapping_mul(17)) * 0.45,
                },
            ));
        }

        for i in 0..ARC_PARTICLES {
            let seed = (i as u32).wrapping_mul(27_811).wrapping_add(59);
            let lane_sign = if hash_to_unit(seed.wrapping_mul(23)) > 0.5 {
                1.0
            } else {
                -1.0
            };
            parent.spawn((
                Name::new("PlasmaBeamParticleArc"),
                Sprite {
                    image: particle_image.clone(),
                    color: Color::srgba(0.86, 0.74, 1.0, 0.92),
                    custom_size: Some(Vec2::splat(PLASMA_ARC_PARTICLE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.14),
                PlasmaBeamParticle {
                    normalized_distance: hash_to_unit(seed.wrapping_mul(3)),
                    lane: lane_sign * (0.45 + hash_to_unit(seed.wrapping_mul(29)) * 0.4),
                    phase: hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU,
                    layer_scale: 0.9 + hash_to_unit(seed.wrapping_mul(31)) * 0.55,
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
    let direction_sign = if beam_direction >= 0.0 { 1.0 } else { -1.0 };
    let clamped_length = beam_length.max(1.0);
    let t = time.elapsed_secs();

    for child_entity in children {
        let Ok((particle, mut transform, mut sprite)) = beam_particles.get_mut(child_entity) else {
            continue;
        };

        let normalized = particle.normalized_distance.clamp(0.0, 1.0);
        let x = direction_sign * clamped_length * normalized;

        let arc_wave = (t * ARC_SPEED + particle.phase + normalized * 17.0).sin() * 0.7
            + (t * (ARC_SPEED * 1.73) + particle.phase * 1.37 + normalized * 31.0).sin() * 0.3;
        let lane_offset = particle.lane * PLASMA_GLOW_WIDTH * particle.layer_scale;
        let y = lane_offset + arc_wave * PLASMA_ARC_AMPLITUDE * particle.layer_scale;

        transform.translation.x = x;
        transform.translation.y = y;

        let front_boost = (0.82 + normalized * 0.18).clamp(0.0, 1.0);
        let flicker = 0.84
            + ((t * 44.0) + particle.phase + normalized * 63.0)
                .sin()
                .abs()
                * 0.22;
        let alpha = (front_boost * flicker * alpha_multiplier).clamp(0.0, 1.0);

        let violet_mix = ((t * 18.0 + particle.phase * 2.2 + normalized * 22.0).sin() * 0.5 + 0.5)
            * 0.55;
        let r = 0.42 + violet_mix * 0.42;
        let g = 0.86 + (1.0 - violet_mix) * 0.12;
        let b = 1.0;
        sprite.color = Color::srgba(r, g, b, alpha);

        let core_bias = (1.0 - particle.lane.abs() * 0.38).clamp(0.72, 1.0);
        let pulse = 0.96 + (t * 23.0 + particle.phase + normalized * 7.0).sin().abs() * 0.22;
        let size = (PLASMA_CORE_WIDTH * particle.layer_scale * core_bias * pulse).max(7.0);
        sprite.custom_size = Some(Vec2::splat(size));
    }

}
