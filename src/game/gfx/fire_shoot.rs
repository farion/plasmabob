use bevy::prelude::*;

use crate::game::gfx::helpers::{hash_to_unit, ProjectileEffectParticle};
use crate::game::gfx::spawn_effect_particles;
use crate::game::runtime_components::GameEntity;
use crate::game::gfx::particles::create_fire_particle_image;

const FIRE_PARTICLE_TEXTURE_SIZE: u32 = 64;

/// Preloaded fire-specific particle image resource.
#[derive(Resource)]
pub(crate) struct FireParticleImage(pub Handle<Image>);

/// Preload the fire particle image into the asset store so it is ready for use.
/// Register this as an `OnEnter(GameView)` startup system.
pub(crate) fn preload_fire_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(create_fire_particle_image(FIRE_PARTICLE_TEXTURE_SIZE));
    commands.insert_resource(FireParticleImage(handle));
}

/// Remove the preloaded fire particle image when the level exits.
pub(crate) fn cleanup_fire_particle_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    fire_res: Option<Res<FireParticleImage>>,
) {
    if let Some(res) = fire_res {
        let handle = res.0.clone();
        images.remove(handle.id());
        commands.remove_resource::<FireParticleImage>();
    }
}

/// Return an existing handle or create and cache the fire particle image on demand.
pub(crate) fn ensure_fire_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }
    let handle = images.add(create_fire_particle_image(FIRE_PARTICLE_TEXTURE_SIZE));
    *local_handle = Some(handle.clone());
    handle
}

/// Spawn fire-projectile particles: a flame-ball made of radial flame tongues.
///
/// Each call emits:
/// - A bright white-yellow hot core cluster
/// - 7 irregular flame tongues (each with 3 particles: base/mid/tip) radiating outward
/// - An orange-red trail streaming behind the flight direction
///
/// Designed to be called every few frames while the projectile is in flight.
pub fn spawn_fire_shoot_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    direction: Vec2,
) {
    // Keep trail visuals aligned with the enlarged fireball silhouette.
    const FIREBALL_VISUAL_SCALE: f32 = 2.0;
    const TRAIL_SCALE: f32 = FIREBALL_VISUAL_SCALE;
    const TRAIL_SPEED_SCALE: f32 = 1.8;
    const TRAIL_LIFETIME_SCALE: f32 = 1.0;

    let dir = if direction.length_squared() > 0.0 {
        direction.normalize()
    } else {
        Vec2::X
    };
    // Hot white-yellow core: larger cluster (stronger yellow) to make the ball bigger.
    // More particles and larger sizes create a visibly larger fireball while keeping the
    // bright center readable.
    for i in 0..6u32 {
        let seed = seed_base.wrapping_add(i.wrapping_mul(997));
        let jitter = Vec2::new(
            (hash_to_unit(seed.wrapping_mul(11)) - 0.5) * 16.0,
            (hash_to_unit(seed.wrapping_mul(13)) - 0.5) * 16.0,
        );
        // Larger core: 36..60
        let size = 36.0 + hash_to_unit(seed.wrapping_mul(17)) * 24.0;
        // Shift color more toward yellow/orange by increasing green and slightly
        // reducing pure red, keeping blue very low.
        let g = 0.72 + hash_to_unit(seed.wrapping_mul(23)) * 0.22;
        let r = 0.92 + hash_to_unit(seed.wrapping_mul(29)) * 0.06;
        let color = Color::srgba(r, g, 0.06, 1.0);
        spawn_flame_particle(
            commands, image,
            origin + jitter, jitter * 0.35,
            size, color, 0.10, z,
            "FireballCore",
        );
    }

    // Flame tongues: 7 irregular arms that radiate outward to form the spiky flame-ball shape.
    // Each tongue has three particles: a large yellow-orange base, an orange mid, and a small
    // deep-orange tip. The varying lengths make the silhouette organic and non-circular.
    const TONGUE_COUNT: u32 = 8;
    for i in 0..TONGUE_COUNT {
        let seed = seed_base
            .wrapping_add(0x3412_AB00)
            .wrapping_add(i.wrapping_mul(1_009));

        // Distribute tongue angles evenly around the ball, plus random jitter
        let base_angle = (i as f32 / TONGUE_COUNT as f32) * std::f32::consts::TAU;
        // stronger angle jitter to make tongues chaotic
        let angle_jitter = (hash_to_unit(seed.wrapping_mul(3)) - 0.5) * 1.4;
        let tongue_dir = Vec2::new(
            (base_angle + angle_jitter).cos(),
            (base_angle + angle_jitter).sin(),
        );
        // Shorter tongues: reduce base length and variance
        let len = 0.35 + hash_to_unit(seed.wrapping_mul(5)) * 0.45;

        // Base of tongue – large, golden-orange, close to the core
        let base_size = 12.0 + hash_to_unit(seed.wrapping_mul(7)) * 6.0;
        // more yellow/orange (increase green), slightly reduce red dominance
        let base_g = 0.72 + hash_to_unit(seed.wrapping_mul(11)) * 0.18;
        let base_r = 0.94 + hash_to_unit(seed.wrapping_mul(31)) * 0.04;
        // add lateral wobble so tongues look irregular
        let lateral = Vec2::new(-tongue_dir.y, tongue_dir.x)
            * ((hash_to_unit(seed.wrapping_mul(37)) - 0.5) * 10.0 * len);
        spawn_flame_particle(
            commands, image,
            origin + tongue_dir * (5.0 * len) + lateral,
            tongue_dir * (10.0 * len) + lateral * 0.25,
            base_size,
            Color::srgba(base_r, base_g, 0.04, 1.0),
            0.095, z - 0.01,
            "FireballTongueBase",
        );

        // Mid of tongue – medium, orange, further out
        let mid_size = 9.0 + hash_to_unit(seed.wrapping_mul(13)) * 4.0;
        let mid_g = 0.58 + hash_to_unit(seed.wrapping_mul(17)) * 0.18;
        let mid_r = 0.94 + hash_to_unit(seed.wrapping_mul(33)) * 0.04;
        spawn_flame_particle(
            commands, image,
            origin + tongue_dir * (12.0 * len) + lateral * 0.5,
            tongue_dir * (14.0 * len) + lateral * 0.15,
            mid_size,
            Color::srgba(mid_r, mid_g, 0.02, 0.98),
            0.09, z - 0.02,
            "FireballTongueMid",
        );

        // Tip of tongue – small, deep orange, outermost
        let tip_size = 6.0 + hash_to_unit(seed.wrapping_mul(19)) * 3.5;
        let tip_g = 0.42 + hash_to_unit(seed.wrapping_mul(23)) * 0.18;
        let tip_r = 0.90 + hash_to_unit(seed.wrapping_mul(35)) * 0.06;
        spawn_flame_particle(
            commands, image,
            origin + tongue_dir * (20.0 * len) + lateral * 0.8,
            tongue_dir * (18.0 * len) + lateral * 0.12,
            tip_size,
            Color::srgba(tip_r, tip_g, 0.01, 0.98),
            0.075, z - 0.03,
            "FireballTongueTip",
        );
    }

    // Add a subtle trail behind the projectile: soft smoke + short-lived sparks.
    // The trail is biased opposite to movement direction and slightly spread sideways.
    let back = -dir;
    let side = Vec2::new(-dir.y, dir.x);

    for i in 0..4u32 {
        let seed = seed_base
            .wrapping_add(0x7821_C000)
            .wrapping_add(i.wrapping_mul(1_237));

        let back_dist = (8.0 + hash_to_unit(seed.wrapping_mul(3)) * 10.0) * TRAIL_SCALE;
        let side_offset = (hash_to_unit(seed.wrapping_mul(5)) - 0.5) * 12.0 * TRAIL_SCALE;
        let jitter = Vec2::new(
            (hash_to_unit(seed.wrapping_mul(7)) - 0.5) * 4.0 * TRAIL_SCALE,
            (hash_to_unit(seed.wrapping_mul(11)) - 0.5) * 4.0 * TRAIL_SCALE,
        );

        let position = origin + back * back_dist + side * side_offset + jitter;
        let velocity = back * (20.0 + hash_to_unit(seed.wrapping_mul(13)) * 35.0) * TRAIL_SPEED_SCALE
            + side * ((hash_to_unit(seed.wrapping_mul(17)) - 0.5) * 14.0) * TRAIL_SPEED_SCALE
            + jitter * 0.3;

        let size = (10.0 + hash_to_unit(seed.wrapping_mul(19)) * 8.0) * TRAIL_SCALE;
        let lifetime = (0.20 + hash_to_unit(seed.wrapping_mul(23)) * 0.12) * TRAIL_LIFETIME_SCALE;
        let color = Color::srgba(
            0.20 + hash_to_unit(seed.wrapping_mul(29)) * 0.10,
            0.20 + hash_to_unit(seed.wrapping_mul(31)) * 0.10,
            0.20 + hash_to_unit(seed.wrapping_mul(37)) * 0.10,
            0.40 + hash_to_unit(seed.wrapping_mul(41)) * 0.18,
        );

        spawn_flame_particle(
            commands,
            image,
            position,
            velocity,
            size,
            color,
            lifetime,
            z - 0.04,
            "FireballTrailSmoke",
        );
    }

    for i in 0..6u32 {
        let seed = seed_base
            .wrapping_add(0x45AA_9000)
            .wrapping_add(i.wrapping_mul(1_409));

        let back_dist = (4.0 + hash_to_unit(seed.wrapping_mul(3)) * 6.0) * TRAIL_SCALE;
        let side_offset = (hash_to_unit(seed.wrapping_mul(5)) - 0.5) * 7.0 * TRAIL_SCALE;
        let jitter = Vec2::new(
            (hash_to_unit(seed.wrapping_mul(7)) - 0.5) * 2.0 * TRAIL_SCALE,
            (hash_to_unit(seed.wrapping_mul(11)) - 0.5) * 2.0 * TRAIL_SCALE,
        );

        let position = origin + back * back_dist + side * side_offset + jitter;
        let velocity = back * (70.0 + hash_to_unit(seed.wrapping_mul(13)) * 80.0) * TRAIL_SPEED_SCALE
            + side * ((hash_to_unit(seed.wrapping_mul(17)) - 0.5) * 50.0) * TRAIL_SPEED_SCALE
            + jitter * 0.8;

        let size = (2.8 + hash_to_unit(seed.wrapping_mul(19)) * 2.8) * TRAIL_SCALE;
        let lifetime = (0.07 + hash_to_unit(seed.wrapping_mul(23)) * 0.06) * TRAIL_LIFETIME_SCALE;
        let color = Color::srgba(
            0.95 + hash_to_unit(seed.wrapping_mul(29)) * 0.05,
            0.55 + hash_to_unit(seed.wrapping_mul(31)) * 0.27,
            0.08 + hash_to_unit(seed.wrapping_mul(37)) * 0.10,
            0.82 + hash_to_unit(seed.wrapping_mul(41)) * 0.16,
        );

        spawn_flame_particle(
            commands,
            image,
            position,
            velocity,
            size,
            color,
            lifetime,
            z - 0.02,
            "FireballTrailSpark",
        );
    }
}

/// Spawn a single `ProjectileEffectParticle` at an offset position from the fireball origin.
fn spawn_flame_particle(
    commands: &mut Commands,
    image: &Handle<Image>,
    position: Vec2,
    velocity: Vec2,
    size: f32,
    color: Color,
    lifetime_secs: f32,
    z: f32,
    name: &'static str,
) {
    commands.spawn((
        Name::new(name),
        Sprite {
            image: image.clone(),
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, z),
        ProjectileEffectParticle {
            velocity,
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
            start_size: size,
            base_color: color,
        },
        GameEntity,
    ));
}

