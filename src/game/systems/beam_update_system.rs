use crate::game::components::plasma::PlasmaBeam;
use crate::game::gfx::helpers::{PlasmaBeamParticle, PlasmaImpactParticle, ProjectileEffectParticle};
use crate::game::gfx::plasma_shoot::update_beam_particles;
use bevy::prelude::*;

pub fn beam_update_system(
    mut commands: Commands,
    time: Res<Time>,
    projectile_transforms: Query<
        &Transform,
        (
            With<crate::game::runtime_components::Projectile>,
            Without<PlasmaBeam>,
        ),
    >,
    mut beams: Query<
        (Entity, &mut PlasmaBeam, &mut Transform, &Children),
        (
            Without<crate::game::runtime_components::Projectile>,
            Without<PlasmaBeamParticle>,
            Without<PlasmaImpactParticle>,
        ),
    >,
    mut beam_particles: Query<
        (&PlasmaBeamParticle, &mut Transform, &mut Sprite),
        (
            Without<PlasmaImpactParticle>,
            Without<crate::game::runtime_components::Projectile>,
            Without<PlasmaBeam>,
        ),
    >,
    mut impact_particles: Query<
        (
            Entity,
            &mut PlasmaImpactParticle,
            &mut Transform,
            &mut Sprite,
        ),
        (
            Without<PlasmaBeamParticle>,
            Without<crate::game::runtime_components::Projectile>,
        ),
    >,
    mut projectile_effect_particles: Query<
        (Entity, &mut ProjectileEffectParticle, &mut Transform, &mut Sprite),
        (
            Without<PlasmaBeamParticle>,
            Without<PlasmaImpactParticle>,
            Without<crate::game::runtime_components::Projectile>,
            Without<PlasmaBeam>,
        ),
    >,
) {
    let dt = time.delta_secs();

    for (beam_entity, mut beam, mut beam_transform, children) in &mut beams {
        let mut alpha_multiplier = 1.0;
        if let Some(target_projectile) = beam.target_projectile {
            if let Ok(projectile_transform) = projectile_transforms.get(target_projectile) {
                let target_pos = projectile_transform.translation.truncate();
                let delta = target_pos - beam.origin;
                if delta.length_squared() > f32::EPSILON {
                    beam.direction = delta.x.signum();
                    beam.current_length = delta.length();
                }
            } else {
                commands.entity(beam_entity).try_despawn();
                continue;
            }
        } else if let Some(lifetime) = beam.lifetime.as_mut() {
            lifetime.tick(time.delta());
            alpha_multiplier = (1.0 - lifetime.fraction()).clamp(0.0, 1.0);
            // use just_finished() which indicates the timer reached its end during the last tick
            if lifetime.just_finished() {
                commands.entity(beam_entity).try_despawn();
                continue;
            }
        }
        beam_transform.translation.x = beam.origin.x;
        beam_transform.translation.y = beam.origin.y;
        // collect child entity ids — call clone on each item which works for both &Entity and Entity
        let child_entities: Vec<Entity> = children.iter().map(|c| c.clone()).collect();
        update_beam_particles(
            &time,
            beam.direction,
            beam.current_length,
            child_entities,
            &mut beam_particles,
            alpha_multiplier,
        );
    }

    for (particle_entity, mut impact, mut transform, mut sprite) in &mut impact_particles {
        impact.lifetime.tick(time.delta());
        transform.translation.x += impact.velocity.x * dt;
        transform.translation.y += impact.velocity.y * dt;
        let fraction = impact.lifetime.fraction();
        let size = (impact.start_size * (1.08 - fraction * 0.55)).max(0.0);
        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color.set_alpha((1.08 - fraction * 0.72).clamp(0.0, 1.0));
        if impact.lifetime.just_finished() {
            commands.entity(particle_entity).try_despawn();
        }
    }

    for (particle_entity, mut particle, mut transform, mut sprite) in &mut projectile_effect_particles {
        particle.lifetime.tick(time.delta());
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let fraction = particle.lifetime.fraction();
        let size = (particle.start_size * (1.0 - fraction * 0.35)).max(0.0);
        sprite.custom_size = Some(Vec2::splat(size));

        let mut color = particle.base_color;
        color.set_alpha((1.0 - fraction * 0.85).clamp(0.0, 1.0));
        sprite.color = color;

        if particle.lifetime.just_finished() {
            commands.entity(particle_entity).try_despawn();
        }
    }
}
