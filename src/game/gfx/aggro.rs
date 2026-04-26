use bevy::prelude::*;

use crate::game::gfx::helpers::{hash_to_unit, ProjectileEffectParticle};
use crate::game::gfx::particles::create_aggro_particle_image;
use crate::game::runtime_components::GameEntity;

const AGGRO_PARTICLE_TEXTURE_SIZE: u32 = 48;

#[derive(Resource)]
pub(crate) struct AggroParticleImage(pub Handle<Image>);

pub(crate) fn preload_aggro_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(create_aggro_particle_image(AGGRO_PARTICLE_TEXTURE_SIZE));
    commands.insert_resource(AggroParticleImage(handle));
}

pub(crate) fn cleanup_aggro_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    aggro_res: Option<Res<AggroParticleImage>>,
) {
    if let Some(res) = aggro_res {
        let handle = res.0.clone();
        images.remove(handle.id());
        commands.remove_resource::<AggroParticleImage>();
    }
}

pub(crate) fn spawn_aggro_sparks(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
) {
    let count = 8 + (hash_to_unit(seed_base.wrapping_mul(3)) * 6.0) as usize;

    for i in 0..count {
        let seed = seed_base
            .wrapping_add((i as u32).wrapping_mul(2_041))
            .wrapping_mul(31_337);

        let angle = hash_to_unit(seed.wrapping_mul(7)) * std::f32::consts::TAU;
        let radial = Vec2::new(angle.cos(), angle.sin());
        let tangent = Vec2::new(-radial.y, radial.x);

        let radius = 8.0 + hash_to_unit(seed.wrapping_mul(11)) * 22.0;
        let jitter = Vec2::new(
            (hash_to_unit(seed.wrapping_mul(13)) - 0.5) * 8.0,
            (hash_to_unit(seed.wrapping_mul(17)) - 0.5) * 8.0,
        );
        let position = origin + radial * radius + jitter;

        // Radial burst plus swirl and slight upward bias to feel like hot sparks.
        let speed = 44.0 + hash_to_unit(seed.wrapping_mul(19)) * 120.0;
        let swirl = (hash_to_unit(seed.wrapping_mul(23)) - 0.5) * 150.0;
        let upward = 24.0 + hash_to_unit(seed.wrapping_mul(29)) * 80.0;
        let velocity = radial * speed + tangent * swirl + Vec2::new(0.0, upward);

        let size = 2.5 + hash_to_unit(seed.wrapping_mul(31)) * 4.4;
        let ttl = 0.16 + hash_to_unit(seed.wrapping_mul(37)) * 0.28;
        let heat = hash_to_unit(seed.wrapping_mul(41));
        let color = Color::srgba(
            0.90 + heat * 0.10,
            0.16 + heat * 0.34,
            0.06 + heat * 0.11,
            0.84 + hash_to_unit(seed.wrapping_mul(43)) * 0.16,
        );

        commands.spawn((
            Name::new("EnemyAggroSpark"),
            Sprite {
                image: image.clone(),
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, z + (i as f32 % 3.0) * 0.01),
            ProjectileEffectParticle {
                velocity,
                lifetime: Timer::from_seconds(ttl, TimerMode::Once),
                start_size: size,
                base_color: color,
            },
            GameEntity,
        ));
    }
}

