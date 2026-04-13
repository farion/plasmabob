use bevy::prelude::*;
use crate::game::gfx::helpers::hash_to_unit;
use crate::game::gfx::spawn_effect_particles;


pub fn spawn_fire_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    direction: Vec2,
) {
    let fire_size_scale = 3.0;

    let dir = if direction.length_squared() > 0.0 {
        direction.normalize()
    } else {
        Vec2::X
    };
    let side = Vec2::new(-dir.y, dir.x);

    // Bright core: a clustered wobbling burst that reads as a compact fireball.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base,
        7,
        |seed| {
            // compute a base velocity vector and scale it so particle spacing increases
            let forward = 16.0 + hash_to_unit(seed.wrapping_mul(13)) * 26.0;
            let lateral_sign = if hash_to_unit(seed.wrapping_mul(17)) > 0.5 { 1.0 } else { -1.0 };
            let lateral = lateral_sign * (8.0 + hash_to_unit(seed.wrapping_mul(19)) * 18.0);
            let wobble = Vec2::new(
                (hash_to_unit(seed.wrapping_mul(23)) - 0.5) * 10.0,
                (hash_to_unit(seed.wrapping_mul(29)) - 0.5) * 10.0,
            );
            (dir * forward + side * lateral + wobble) * fire_size_scale
        },
        |seed| (7.0 + hash_to_unit(seed.wrapping_mul(31)) * 5.5) * fire_size_scale,
        |seed| {
            let heat = hash_to_unit(seed.wrapping_mul(37));
            let r = 0.95 + heat * 0.05;
            let g = 0.35 + heat * 0.45;
            let b = 0.05 + heat * 0.08;
            Color::srgba(r, g, b, 1.0)
        },
        0.42,
    );

    // Short spark trail: mostly behind projectile movement, but with wild spread.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base.wrapping_add(9_731),
        6,
        |seed| {
            // spark trail: place sparks further out by scaling their velocities
            let back = 62.0 + hash_to_unit(seed.wrapping_mul(41)) * 95.0;
            let lateral_sign = if hash_to_unit(seed.wrapping_mul(43)) > 0.5 { 1.0 } else { -1.0 };
            let lateral = lateral_sign * (18.0 + hash_to_unit(seed.wrapping_mul(47)) * 44.0);
            let drift = dir * (hash_to_unit(seed.wrapping_mul(53)) * 20.0);
            ((-dir * back) + (side * lateral) + drift) * fire_size_scale
        },
        |seed| (2.8 + hash_to_unit(seed.wrapping_mul(59)) * 3.0) * fire_size_scale,
        |seed| {
            let glow = hash_to_unit(seed.wrapping_mul(61));
            let r = 0.95 + glow * 0.05;
            let g = 0.55 + glow * 0.4;
            let b = 0.12 + glow * 0.08;
            Color::srgba(r, g, b, 1.0)
        },
        0.24,
    );

}

