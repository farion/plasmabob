use bevy::prelude::*;
use avian2d::prelude::RigidBody;
use crate::MainCamera;

const PARALLAX_BACKGROUND_SPEED: f32 = 0.08;
const PARALLAX_MIN_SPEED: f32 = 0.12;
const PARALLAX_MAX_SPEED: f32 = 1.5;
const PARALLAX_MIN_Z: f32 = 0.0;
const PARALLAX_MAX_Z: f32 = 150.0;
// Only apply parallax for entities outside the "no-parallax" middle band.
const PARALLAX_NO_EFFECT_LOWER_Z: f32 = 75.0;
const PARALLAX_NO_EFFECT_UPPER_Z: f32 = 125.0;

#[derive(Component)]
pub(crate) struct BackgroundParallax;

#[derive(Component)]
pub(crate) struct ParallaxAnchor {
    base_x: f32,
    speed: f32,
}

pub(crate) fn attach_parallax_anchors(
    mut commands: Commands,
    cameras: Query<&Transform, (With<Camera>, With<MainCamera>)>,
    // Consider newly spawned level entities: we only want to attach anchors to
    // purely visual entities (have Sprite) that do NOT have a RigidBody, so we
    // don't overwrite physics-driven objects (NPCs, dynamic platforms).
    spawned_visuals: Query<(
        Entity,
        &Transform,
        Option<&Sprite>,
        Option<&RigidBody>,
    ), (With<crate::game::components::SpawnedLevelEntity>, Without<ParallaxAnchor>, Added<crate::game::components::SpawnedLevelEntity>)>,
    backgrounds: Query<(Entity, &Transform), (With<BackgroundParallax>, Without<ParallaxAnchor>, Added<BackgroundParallax>)>,
) {
    // Determine current camera x (if available). If no camera is present yet, assume 0.
    let camera_x = cameras.single().map(|t| t.translation.x).unwrap_or(0.0);

    for (entity, transform, sprite_opt, rb_opt) in &spawned_visuals {
        // Only attach to entities that have a Sprite and no RigidBody
        if sprite_opt.is_none() || rb_opt.is_some() {
            continue;
        }

        let z = transform.translation.z;
        // Apply parallax only for z < PARALLAX_NO_EFFECT_LOWER_Z or z > PARALLAX_NO_EFFECT_UPPER_Z
        if !(z < PARALLAX_NO_EFFECT_LOWER_Z || z > PARALLAX_NO_EFFECT_UPPER_Z) {
            continue;
        }

        let speed = parallax_speed_from_z(z);
        // Store base_x such that parallax_world_x(base_x, camera_x_at_attach, speed) == transform.translation.x
        // i.e. base_x = current_x - camera_x * (1.0 - speed)
        let base_x = transform.translation.x - camera_x * (1.0 - speed);
        commands.entity(entity).insert(ParallaxAnchor { base_x, speed });
    }

    for (entity, transform) in &backgrounds {
        let speed = PARALLAX_BACKGROUND_SPEED;
        let base_x = transform.translation.x - camera_x * (1.0 - speed);
        commands.entity(entity).insert(ParallaxAnchor { base_x, speed });
    }
}

pub(crate) fn apply_parallax_from_camera(
    cameras: Query<&Transform, (With<Camera>, With<MainCamera>)>,
    mut entities: Query<(&ParallaxAnchor, &mut Transform), Without<MainCamera>>,
) {
    let Ok(camera_transform) = cameras.single() else {
        return;
    };

    let camera_x = camera_transform.translation.x;

    for (anchor, mut transform) in &mut entities {
        transform.translation.x = parallax_world_x(anchor.base_x, camera_x, anchor.speed);
    }
}

fn parallax_world_x(base_x: f32, camera_x: f32, speed: f32) -> f32 {
    base_x + camera_x * (1.0 - speed)
}

fn parallax_speed_from_z(z_index: f32) -> f32 {
    let normalized = ((z_index - PARALLAX_MIN_Z) / (PARALLAX_MAX_Z - PARALLAX_MIN_Z)).clamp(0.0, 1.0);
    PARALLAX_MIN_SPEED + normalized * (PARALLAX_MAX_SPEED - PARALLAX_MIN_SPEED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallax_speed_increases_with_z() {
        let slow = parallax_speed_from_z(-50.0);
        let medium = parallax_speed_from_z(50.0);
        let fast = parallax_speed_from_z(150.0);

        assert!(slow < medium);
        assert!(medium < fast);
    }

    #[test]
    fn background_speed_is_slower_than_world_speed() {
        assert!(PARALLAX_BACKGROUND_SPEED < 1.0);
    }

    #[test]
    fn parallax_world_x_moves_faster_for_higher_speed() {
        let base_x = 1000.0;
        let camera_x = 200.0;

        let slow_world_x = parallax_world_x(base_x, camera_x, 0.3);
        let fast_world_x = parallax_world_x(base_x, camera_x, 1.3);

        assert!(slow_world_x > fast_world_x);
    }
}


