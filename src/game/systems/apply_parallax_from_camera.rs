use bevy::prelude::*;

use crate::MainCamera;
use crate::game::systems::parallax_types::ParallaxAnchor;

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
