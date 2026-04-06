use bevy::prelude::*;
use crate::game::gfx::spawn_effect_particles;
use crate::game::gfx::helpers::hash_to_unit;

pub fn spawn_spit_particles(
    commands: &mut Commands,
    image: &Handle<Image>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
) {
    spawn_effect_particles(
        commands,
        image,
        origin,
        z,
        seed_base,
        5,
        |seed| {
            let angle = hash_to_unit(seed.wrapping_mul(43)) * std::f32::consts::TAU;
            let speed = 30.0 + hash_to_unit(seed.wrapping_mul(67)) * 50.0;
            Vec2::new(angle.cos(), angle.sin()) * speed
        },
        |seed| 4.0 + hash_to_unit(seed.wrapping_mul(23)) * 5.0,
        |seed| {
            let r = 0.85 + hash_to_unit(seed.wrapping_mul(79)) * 0.15;
            let g = 0.6 + hash_to_unit(seed.wrapping_mul(83)) * 0.2;
            Color::srgba(r, g, 0.1, 1.0)
        },
        0.38,
    );
}

