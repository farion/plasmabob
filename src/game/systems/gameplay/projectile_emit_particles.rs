use bevy::prelude::*;

use crate::game::systems::gameplay::types::{ProjectileEmitter, ProjectileParticleKind, RangeProjectile};
use crate::helper::particles::create_round_particle_image;
use crate::game::gfx::{fire, poison, spit};

pub(crate) fn projectile_emit_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut particle_image: Local<Option<Handle<Image>>>,
    mut emitters: Query<(&Transform, &mut ProjectileEmitter, &crate::game::systems::gameplay::types::RangeProjectile), With<RangeProjectile>>,
) {
    // Lazily create a shared round soft-circle image for all effect particles.
    let img = match particle_image.as_ref() {
        Some(h) => h.clone(),
        None => {
            let h = images.add(create_round_particle_image(24));
            *particle_image = Some(h.clone());
            h
        }
    };

    for (transform, mut emitter, projectile) in &mut emitters {
        emitter.timer.tick(time.delta());
        if !emitter.timer.just_finished() {
            continue;
        }

        let pos = transform.translation;
        let z = pos.z + 1.0; // above the projectile sprite
        let seed_base = (pos.x.to_bits() ^ pos.y.to_bits()) as u32;
        // projectile velocity (used to orient bursts and inherit movement)
        let proj_velocity = projectile.velocity;

        match emitter.kind {
            ProjectileParticleKind::Fire => {
                // fire expects a direction-like vector; passing full velocity is fine (it will normalize internally)
                fire::spawn_fire_particles(&mut commands, &img, Vec2::new(pos.x, pos.y), z, seed_base, proj_velocity);
            }
            ProjectileParticleKind::Poison => {
                poison::spawn_poison_particles(&mut commands, &img, Vec2::new(pos.x, pos.y), z, seed_base, proj_velocity);
            }
            ProjectileParticleKind::Spit => {
                spit::spawn_spit_particles(&mut commands, &img, Vec2::new(pos.x, pos.y), z, seed_base);
            }
        }
    }
}


