use bevy::prelude::*;
use crate::game::systems::gameplay::types::DustParticle;

pub(crate) fn update_dust_particles(
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

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

