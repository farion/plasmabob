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
        transform.translation.x = crate::game::systems::common::parallax_helpers::parallax_world_x(anchor.base_x, camera_x, anchor.speed);
    }
}

