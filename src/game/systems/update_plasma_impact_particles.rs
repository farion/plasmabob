use bevy::prelude::*;

use crate::game::systems::combat_types::PlasmaImpactParticle;

pub(crate) fn update_plasma_impact_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut PlasmaImpactParticle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();
        particle.velocity *= 0.86;

        particle.lifetime.tick(time.delta());
        let remaining = 1.0 - particle.lifetime.fraction();

        sprite.color = Color::srgba(0.25, 0.95, 1.0, remaining.clamp(0.0, 1.0));
        let size = particle.start_size * (0.6 + remaining.max(0.0));
        sprite.custom_size = Some(Vec2::splat(size));

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

