use bevy::prelude::*;

use crate::game::view_api::GameViewEntity;

use crate::game::systems::common::combat_helpers::{hash_to_unit, create_round_particle_image};

pub(crate) fn dust_origin(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    Vec2::new(transform.translation.x, transform.translation.y - (size.y * 0.45))
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
            crate::game::systems::player_types::DustParticle {
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

