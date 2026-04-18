use bevy::prelude::*;
use crate::game::runtime_components::{Parallax, ParallaxCameraOrigin};

/// Apply parallax offsets on both X and Y based on camera movement.
pub fn apply_parallax_system(
    debug_settings: Res<crate::DebugRenderSettings>,
    camera_origin: Option<Res<ParallaxCameraOrigin>>,
    camera_query: Query<&Transform, With<crate::MainCamera>>,
    mut query: Query<(&Parallax, &mut Transform), Without<crate::MainCamera>>,
) {
    let Some(camera_origin) = camera_origin else {
        return;
    };
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };

    let camera_delta = camera_tf.translation.truncate() - camera_origin.as_ref().0;

    for (parallax, mut transform) in &mut query {
        let target = if debug_settings.as_ref().parallax_enabled {
            parallax.base_position + camera_delta * (1.0 - parallax.factor)
        } else {
            parallax.base_position
        };

        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
}
