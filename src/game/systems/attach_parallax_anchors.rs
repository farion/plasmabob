use bevy::prelude::*;
use avian2d::prelude::RigidBody;

use crate::MainCamera;
use crate::game::components::SpawnedLevelEntity;
use crate::game::systems::parallax_types::{ParallaxAnchor, BackgroundParallax};

const PARALLAX_BACKGROUND_SPEED: f32 = 0.08;
const PARALLAX_MIN_SPEED: f32 = 0.12;
const PARALLAX_MAX_SPEED: f32 = 1.5;
const PARALLAX_MIN_Z: f32 = 0.0;
const PARALLAX_MAX_Z: f32 = 150.0;
const PARALLAX_NO_EFFECT_LOWER_Z: f32 = 75.0;
const PARALLAX_NO_EFFECT_UPPER_Z: f32 = 125.0;

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
    ), (With<SpawnedLevelEntity>, Without<ParallaxAnchor>, Added<SpawnedLevelEntity>)>,
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



fn parallax_speed_from_z(z_index: f32) -> f32 {
    let normalized = ((z_index - PARALLAX_MIN_Z) / (PARALLAX_MAX_Z - PARALLAX_MIN_Z)).clamp(0.0, 1.0);
    PARALLAX_MIN_SPEED + normalized * (PARALLAX_MAX_SPEED - PARALLAX_MIN_SPEED)
}

