use crate::game::gfx::helpers::hash_to_unit;
use crate::game::gfx::spawn_effect_particles;
use bevy::prelude::*;

pub fn spawn_poison_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    projectile_velocity: Vec2,
) {
    // Determine a local forward direction from the projectile velocity so
    // we can create coherent wobble around the projectile's movement.
    let dir = if projectile_velocity.length_squared() > 0.0 {
        projectile_velocity.normalize()
    } else {
        Vec2::X
    };
    let side = Vec2::new(-dir.y, dir.x);
    // Scale factor to make the whole poisonous ball larger while preserving
    // relative ratios and spacing between particles.
    let size_scale = 4.0;

    // Core: a compact, wobbling poisonous ball that largely follows the
    // projectile_velocity but has internal jitter so it reads as a single
    // wobbling blob.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base,
        9,
        |seed| {
            // small local offsets produce the wobble – these are added ON TOP
            // of the projectile_velocity so the whole cluster moves together.
            let lateral = (hash_to_unit(seed.wrapping_mul(11)) - 0.5) * 10.0;
            let vertical = (hash_to_unit(seed.wrapping_mul(13)) - 0.5) * 10.0;
            let jitter = Vec2::new(lateral, vertical);

            // Slight expansion from the center so particles don't perfectly
            // overlap; scale down a touch so the blob remains compact.
            // Apply size scaling to the generated local motion so the whole
            // cluster becomes `size_scale` times larger while the base
            // projectile_velocity stays as the group's global motion.
            projectile_velocity
                + (dir * (hash_to_unit(seed.wrapping_mul(17)) - 0.5) * 6.0 * size_scale)
                + (side * lateral * size_scale)
                + jitter * 0.6 * size_scale
        },
        |seed| (6.0 + hash_to_unit(seed.wrapping_mul(19)) * 6.0) * size_scale,
        |seed| {
            // green, slightly sickly colors with a small alpha variance
            let g = 0.6 + hash_to_unit(seed.wrapping_mul(23)) * 0.35;
            let b = 0.05 + hash_to_unit(seed.wrapping_mul(29)) * 0.15;
            let r = 0.05 + hash_to_unit(seed.wrapping_mul(31)) * 0.12;
            Color::srgba(r, g, b, 0.95)
        },
        0.72,
    );

    // Trail / droplets: a larger set of smaller particles that lag behind
    // the main ball and create a dripping trail. These also include
    // projectile_velocity so the entire effect keeps the projectile motion.
    // Increased particle count to make the effect drip much more.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base.wrapping_add(7_333),
        18,
        |seed| {
            let back = 18.0 + hash_to_unit(seed.wrapping_mul(37)) * 36.0;
            let lateral = (hash_to_unit(seed.wrapping_mul(41)) - 0.5) * 22.0;
            let wobble = Vec2::new(
                (hash_to_unit(seed.wrapping_mul(43)) - 0.5) * 8.0,
                (hash_to_unit(seed.wrapping_mul(47)) - 0.5) * 8.0,
            );

            // place these mostly behind the projectile movement, but add the
            // projectile_velocity so the group inherits the same global motion.
            projectile_velocity * 0.35
                + ((-dir * back) * size_scale)
                + (side * lateral * size_scale)
                + wobble * size_scale
        },
        |seed| (2.2 + hash_to_unit(seed.wrapping_mul(53)) * 3.2) * size_scale,
        |seed| {
            let glow = hash_to_unit(seed.wrapping_mul(59));
            // slightly more translucent for the trail
            let g = 0.55 + glow * 0.35;
            let b = 0.08 + glow * 0.12;
            Color::srgba(0.06, g, b, 0.85)
        },
        0.44,
    );

    // Dense droplet field: many small droplets with shorter lifetime to give
    // the impression of dripping/atomized poison around the projectile.
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base.wrapping_add(13_579),
        24,
        |seed| {
            // lots of small random directions around the projectile with a
            // bias to fall behind the projectile (dripping effect). Scale
            // motions with size_scale so distances stay coherent with the
            // enlarged main ball.
            let angle = hash_to_unit(seed.wrapping_mul(71)) * std::f32::consts::TAU;
            let spread = 18.0 + hash_to_unit(seed.wrapping_mul(73)) * 48.0;
            let dir_rand = Vec2::new(angle.cos(), angle.sin()) * spread * size_scale * 0.6;
            let back_bias =
                -dir * (12.0 + hash_to_unit(seed.wrapping_mul(79)) * 40.0) * size_scale * 0.6;

            // keep some of the projectile velocity so droplets move with it
            projectile_velocity * 0.2 + dir_rand + back_bias
        },
        |seed| (0.8 + hash_to_unit(seed.wrapping_mul(83)) * 1.8) * (size_scale * 0.45),
        |seed| {
            let g = 0.55 + hash_to_unit(seed.wrapping_mul(89)) * 0.4;
            let b = 0.06 + hash_to_unit(seed.wrapping_mul(97)) * 0.12;
            Color::srgba(0.04, g, b, 0.85)
        },
        0.38,
    );
}
