use bevy::prelude::*;
use crate::game::runtime_components::{GameEntity, Parallax, ParallaxCameraOrigin};

const GAMEPLAY_Z_MIN: f32 = 75.0;
const GAMEPLAY_Z_MAX: f32 = 125.0;

/// Capture camera origin and mark non-gameplay entities for parallax.
pub fn init_parallax_system(
    mut commands: Commands,
    camera_query: Query<&Transform, With<crate::MainCamera>>,
    entities: Query<
        (Entity, &Transform),
        (With<GameEntity>, Without<Parallax>, Without<crate::MainCamera>),
    >,
) {
    let Ok(camera_tf) = camera_query.single() else {
        tracing::warn!("init_parallax_system: missing main camera");
        return;
    };
    commands.insert_resource(ParallaxCameraOrigin(camera_tf.translation.truncate()));
    for (entity, transform) in &entities {
        let z = transform.translation.z;
        if (GAMEPLAY_Z_MIN..=GAMEPLAY_Z_MAX).contains(&z) {
            continue;
        }
        let factor = parallax_factor_for_z(z);
        commands.entity(entity).insert(Parallax {
            base_position: transform.translation.truncate(),
            factor,
        });
    }
}

fn parallax_factor_for_z(z: f32) -> f32 {
    if z < GAMEPLAY_Z_MIN {
        // World/background range [0..74] maps to slower-than-standard [0.20..0.98].
        let t = (z.clamp(0.0, 74.0)) / 74.0;
        return 0.20 + t * (0.98 - 0.20);
    }
    if z > GAMEPLAY_Z_MAX {
        // Foreground range [126..250+] maps to faster-than-standard [1.02..1.80].
        let t = ((z - 126.0) / (250.0 - 126.0)).clamp(0.0, 1.0);
        return 1.02 + t * (1.80 - 1.02);
    }
    1.0
}
