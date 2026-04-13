use crate::game::components::plasma::PlasmaBeam;
use crate::game::gfx::helpers::{PlasmaBeamParticle, PlasmaImpactParticle};
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
) {
    let dt = time.delta_secs();

    for (beam_entity, mut beam, mut beam_transform, children) in &mut beams {
        if let Some(target_projectile) = beam.target_projectile {
            if let Ok(projectile_transform) = projectile_transforms.get(target_projectile) {
                let target_pos = projectile_transform.translation.truncate();
                let delta = target_pos - beam.origin;
                if delta.length_squared() > f32::EPSILON {
                    beam.direction = delta.x.signum();
                    beam.current_length = delta.length();
                }
            } else {
                commands.entity(beam_entity).despawn();
                continue;
            }
        } else if let Some(lifetime) = beam.lifetime.as_mut() {
            lifetime.tick(time.delta());
            // use just_finished() which indicates the timer reached its end during the last tick
            if lifetime.just_finished() {
                commands.entity(beam_entity).despawn();
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
            1.0,
        );
    }

    for (particle_entity, mut impact, mut transform, mut sprite) in &mut impact_particles {
        impact.lifetime.tick(time.delta());
        transform.translation.x += impact.velocity.x * dt;
        transform.translation.y += impact.velocity.y * dt;
        let fraction = impact.lifetime.fraction();
        let size = (impact.start_size * (1.0 - fraction * 0.8)).max(0.0);
        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color.set_alpha((1.0 - fraction).clamp(0.0, 1.0));
        if impact.lifetime.just_finished() {
            commands.entity(particle_entity).despawn();
        }
    }
}
