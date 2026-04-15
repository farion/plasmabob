use crate::game::gfx::helpers::hash_to_unit;
use crate::game::gfx::spawn_effect_particles;
use bevy::prelude::*;

pub fn spawn_fire_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    direction: Vec2,
) {
    let fire_size_scale = 4.2;
    // Trail particles should be noticeably smaller than core particles.
    // Make them much smaller so the trail reads as many small flickering embers.
    let trail_size_scale = fire_size_scale * 0.12;

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
        9,
        |seed| {
            // compute a base velocity vector and scale it so particle spacing increases
            let forward = 10.0 + hash_to_unit(seed.wrapping_mul(13)) * 18.0;
            let lateral_sign = if hash_to_unit(seed.wrapping_mul(17)) > 0.5 {
                1.0
            } else {
                -1.0
            };
            let lateral = lateral_sign * (10.0 + hash_to_unit(seed.wrapping_mul(19)) * 22.0);
            let wobble = Vec2::new(
                (hash_to_unit(seed.wrapping_mul(23)) - 0.5) * 18.0,
                (hash_to_unit(seed.wrapping_mul(29)) - 0.5) * 18.0,
            );
            (dir * forward + side * lateral + wobble) * fire_size_scale
        },
        |seed| (9.5 + hash_to_unit(seed.wrapping_mul(31)) * 8.5) * fire_size_scale,
        |seed| {
            let heat = hash_to_unit(seed.wrapping_mul(37));
            let r = 0.95 + heat * 0.05;
            let g = 0.35 + heat * 0.45;
            let b = 0.05 + heat * 0.08;
            Color::srgba(r, g, b, 1.0)
        },
        0.34,
    );

    // Flame-like trail: emit particles inside an upward-biased cone so the
    // trail forms a single flame plume (not two symmetric lobes). Use many
    // very small particles with strong jitter.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base.wrapping_add(9_731),
        24,
        |seed| {
            // single-plume construction: base behind + strong upward bias
            let back = 4.0 + hash_to_unit(seed.wrapping_mul(41)) * 10.0;
            let up = 6.0 + hash_to_unit(seed.wrapping_mul(43)) * 18.0;
            let base_vec = -dir * back + Vec2::new(0.0, up);
            // Remove any symmetric lateral component to guarantee exactly one plume.
            let lateral = Vec2::ZERO;
            // micro-jitter: very small horizontal jitter and strictly positive vertical jitter
            let jitter = Vec2::new(
                (hash_to_unit(seed.wrapping_mul(49)) - 0.5) * 4.0,
                hash_to_unit(seed.wrapping_mul(51)) * 12.0,
            );
            (base_vec + lateral + jitter) * trail_size_scale
        },
        // particle size: very small
        |seed| (0.02 + hash_to_unit(seed.wrapping_mul(59)) * 0.12) * trail_size_scale,
        |seed| {
            // color: orange-yellow with alpha variation
            let g = 0.22 + hash_to_unit(seed.wrapping_mul(61)) * 0.48;
            let a = 0.65 + hash_to_unit(seed.wrapping_mul(63)) * 0.25;
            Color::srgba(1.0, g, 0.06, a)
        },
        0.06,
    );
}
