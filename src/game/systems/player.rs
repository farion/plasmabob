use avian2d::prelude::{Collider, LinearVelocity, ShapeCaster, ShapeHits};
use bevy::ecs::query::Has;
use bevy::math::Dir2;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer, can_set_state};
use crate::game::components::health::Health;
use crate::game::components::hitbox::PrecomputedPlayerHitbox;
use crate::game::components::moving::Moving;
use crate::game::components::player::Player;

use super::{GameViewEntity, Grounded, PLAYER_JUMP_SPEED, PLAYER_MOVE_SPEED};

const JUMP_DUST_COUNT: usize = 8;
const LAND_DUST_COUNT: usize = 12;
const DUST_LIFETIME_SECS: f32 = 0.28;
const DUST_Z: f32 = 9.0;

#[derive(Component)]
pub(super) struct DustParticle {
    velocity: Vec2,
    lifetime: Timer,
    start_size: f32,
}

pub(super) fn configure_player_controller(
    mut commands: Commands,
    mut players: Query<(Entity, &PrecomputedPlayerHitbox), (With<Player>, Without<ShapeCaster>)>,
) {
    for (player, precomputed_hitbox) in &mut players {
        commands.entity(player).insert(
            ShapeCaster::new(
                precomputed_hitbox.ground_caster(false),
                Vec2::ZERO,
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(8.0),
        );
    }
}

pub(super) fn update_grounded(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut dust_particle_image: Local<Option<Handle<Image>>>,
    players: Query<(Entity, Has<Grounded>, &ShapeHits, &Transform, &Sprite), (With<Player>, With<ShapeCaster>)>,
) {
    let particle_image = ensure_dust_particle_image(&mut dust_particle_image, &mut images);

    for (player, was_grounded, hits, transform, sprite) in &players {
        let is_grounded = !hits.is_empty();

        if is_grounded && !was_grounded {
            spawn_dust_burst(
                &mut commands,
                dust_origin(transform, sprite),
                &particle_image,
                LAND_DUST_COUNT,
                player.index() as u32 + 100,
                220.0,
            );
        }

        if is_grounded {
            commands.entity(player).insert(Grounded);
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
}

pub(super) fn control_player(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut dust_particle_image: Local<Option<Handle<Image>>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<
        (
            Entity,
            &mut LinearVelocity,
            Has<Grounded>,
            &Transform,
            &mut Sprite,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&Health>,
        ),
        With<Player>,
    >,
) {
    let particle_image = ensure_dust_particle_image(&mut dust_particle_image, &mut images);

    for (entity, mut velocity, is_grounded, transform, mut sprite, mut state, hit_timer, health) in &mut players {
        if health.is_some_and(|value| value.is_dead()) {
            velocity.x = 0.0;
            continue;
        }

        let mut move_axis = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) {
            move_axis -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            move_axis += 1.0;
        }

        velocity.x = move_axis * PLAYER_MOVE_SPEED;
        update_sprite_flip_for_move_axis(&mut sprite, move_axis);

        if keys.just_pressed(KeyCode::ArrowUp) && is_grounded {
            velocity.y = PLAYER_JUMP_SPEED;
            spawn_dust_burst(
                &mut commands,
                dust_origin(transform, &sprite),
                &particle_image,
                JUMP_DUST_COUNT,
                entity.index() as u32 + 1_000,
                180.0,
            );
        }

        let next_state = if !is_grounded {
            EntityState::Jump
        } else if move_axis.abs() > f32::EPSILON {
            EntityState::Walk
        } else {
            EntityState::Default
        };

        if can_set_state(&state, hit_timer, next_state) {
            state.set(next_state);
        }
    }
}

pub(super) fn update_dust_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut DustParticle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        particle.velocity.x *= 0.9;
        particle.velocity.y = (particle.velocity.y - 520.0 * time.delta_secs()).max(-120.0);

        particle.lifetime.tick(time.delta());
        let remaining = 1.0 - particle.lifetime.fraction();
        let alpha = (remaining * 0.65).clamp(0.0, 1.0);
        let size = particle.start_size * (0.7 + remaining * 0.6);

        sprite.color = Color::srgba(0.55, 0.55, 0.55, alpha);
        sprite.custom_size = Some(Vec2::splat(size));

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn sync_player_hitbox_orientation(
    mut players: Query<
        (&Sprite, &PrecomputedPlayerHitbox, &mut Collider, Option<&mut ShapeCaster>),
        (Or<(With<Player>, With<Moving>)>, Changed<Sprite>),
    >,
) {
    for (sprite, precomputed_hitbox, mut collider, shape_caster) in &mut players {
        *collider = precomputed_hitbox.collider(sprite.flip_x);

        if let Some(mut shape_caster) = shape_caster {
            shape_caster.shape = precomputed_hitbox.ground_caster(sprite.flip_x);
        }
    }
}

fn update_sprite_flip_for_move_axis(sprite: &mut Sprite, move_axis: f32) {
    if move_axis < 0.0 && !sprite.flip_x {
        sprite.flip_x = true;
    } else if move_axis > 0.0 && sprite.flip_x {
        sprite.flip_x = false;
    }
}

fn spawn_dust_burst(
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
            Transform::from_xyz(origin.x, origin.y, DUST_Z),
            DustParticle {
                velocity: Vec2::new(horizontal, upward),
                lifetime: Timer::from_seconds(DUST_LIFETIME_SECS, TimerMode::Once),
                start_size: size,
            },
            GameViewEntity,
        ));
    }
}

fn dust_origin(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    Vec2::new(transform.translation.x, transform.translation.y - (size.y * 0.45))
}

fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}

fn ensure_dust_particle_image(
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

fn create_round_particle_image(size: u32) -> Image {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::hitbox::PolygonHitbox;

    #[test]
    fn flips_sprite_when_moving_left() {
        let mut sprite = Sprite::default();

        update_sprite_flip_for_move_axis(&mut sprite, -1.0);

        assert!(sprite.flip_x);
    }

    #[test]
    fn unflips_sprite_when_moving_right() {
        let mut sprite = Sprite {
            flip_x: true,
            ..default()
        };

        update_sprite_flip_for_move_axis(&mut sprite, 1.0);

        assert!(!sprite.flip_x);
    }

    #[test]
    fn keeps_last_sprite_direction_while_idle() {
        let mut sprite = Sprite {
            flip_x: true,
            ..default()
        };

        update_sprite_flip_for_move_axis(&mut sprite, 0.0);

        assert!(sprite.flip_x);
    }

    #[test]
    fn mirrors_hitbox_points_when_sprite_is_flipped() {
        let polygon_hitbox = PolygonHitbox {
            points: vec![
                Vec2::new(-2.0, -1.0),
                Vec2::new(3.0, -1.0),
                Vec2::new(1.0, 4.0),
            ],
        };

        let mirrored = polygon_hitbox.effective_points(true);

        assert_eq!(
            mirrored,
            vec![
                Vec2::new(-1.0, 4.0),
                Vec2::new(-3.0, -1.0),
                Vec2::new(2.0, -1.0),
            ]
        );
    }

}



