use bevy::prelude::*;

use crate::game::view_api::DebugStatsLabel;

use crate::game::systems::common::debug_helpers::build_stats_text;

pub(crate) fn update_debug_stats_labels(
    mut commands: Commands,
    debug_settings: Res<crate::DebugRenderSettings>,
    mut labels: Query<(Entity, &DebugStatsLabel, &mut Transform, &mut Text2d)>,
    targets: Query<(
        &GlobalTransform,
        Option<&crate::game::components::health::Health>,
        Option<&crate::game::components::health::Damage>,
        Option<&crate::game::components::player::PlasmaAttack>,
        Option<&crate::game::components::animation::AnimationState>,
    )>,
) {
    if !debug_settings.show_hitbox_lines {
        return;
    }

    for (label_entity, label, mut transform, mut text) in &mut labels {
        match targets.get(label.target) {
            Ok((target_transform, health, damage, plasma, state)) => {
                let pos = target_transform.translation();
                transform.translation = Vec3::new(pos.x, pos.y + 80.0, 100.0);
                *text = Text2d::new(build_stats_text(health, damage, plasma, state));
            }
            Err(_) => {
                commands.entity(label_entity).despawn();
            }
        }
    }
}

