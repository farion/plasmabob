use bevy::prelude::*;

use crate::game::components::doodad::Doodad;
use crate::MainCamera;

const PARALLAX_BACKGROUND_SPEED: f32 = 0.08;
const PARALLAX_MIN_SPEED: f32 = 0.12;
const PARALLAX_MAX_SPEED: f32 = 1.5;
const PARALLAX_MIN_Z: f32 = 0.0;
const PARALLAX_MAX_Z: f32 = 150.0;

#[derive(Component)]
pub(super) struct BackgroundParallax;

#[derive(Component)]
pub(super) struct ParallaxAnchor {
    base_x: f32,
    speed: f32,
}

pub(super) fn attach_parallax_anchors(
    mut commands: Commands,
    doodads: Query<(Entity, &Transform), (With<Doodad>, Without<ParallaxAnchor>, Added<Doodad>)>,
    backgrounds: Query<(Entity, &Transform), (With<BackgroundParallax>, Without<ParallaxAnchor>, Added<BackgroundParallax>)>,
) {
    for (entity, transform) in &doodads {
        commands.entity(entity).insert(ParallaxAnchor {
            base_x: transform.translation.x,
            speed: parallax_speed_from_z(transform.translation.z),
        });
    }

    for (entity, transform) in &backgrounds {
        commands.entity(entity).insert(ParallaxAnchor {
            base_x: transform.translation.x,
            speed: PARALLAX_BACKGROUND_SPEED,
        });
    }
}

pub(super) fn apply_parallax_from_camera(
    cameras: Query<&Transform, (With<Camera>, With<MainCamera>)>,
    mut entities: Query<(&ParallaxAnchor, &mut Transform), Without<MainCamera>>,
) {
    let Ok(camera_transform) = cameras.get_single() else {
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


