use bevy::prelude::*;

use crate::game::systems::gameplay::types::ProjectileEffectParticle;

pub(crate) fn update_projectile_effect_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut ProjectileEffectParticle, &mut Transform, &mut Sprite)>,
) {
    let delta = time.delta_secs();

    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        // Move particle.
        transform.translation.x += particle.velocity.x * delta;
        transform.translation.y += particle.velocity.y * delta;

        // Apply friction (0.88 per frame at 60 fps).
        let friction = 0.88_f32.powf(delta * 60.0);
        particle.velocity *= friction;

        // Tick lifetime.
        particle.lifetime.tick(time.delta());
        let remaining = 1.0 - particle.lifetime.fraction();
        let alpha = (remaining * 0.9).clamp(0.0, 1.0);

        // Modulate alpha while keeping base_color hue/saturation/brightness.
        let base = particle.base_color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, alpha);
        sprite.custom_size = Some(Vec2::splat(particle.start_size * (0.35 + remaining * 0.65)));

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

