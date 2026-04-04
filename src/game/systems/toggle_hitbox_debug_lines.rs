use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::view_api::{DebugStatsLabel, GameViewEntity};

use crate::game::systems::debug_helpers::{toggle_hitbox_lines, build_stats_text};

pub(crate) fn toggle_hitbox_debug_lines(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
    label_query: Query<Entity, With<DebugStatsLabel>>,
    entity_query: Query<(
        Entity,
        Option<&crate::game::components::health::Health>,
        Option<&crate::game::components::health::Damage>,
        Option<&crate::game::components::player::PlasmaAttack>,
        Option<&crate::game::components::animation::AnimationState>,
    ), With<SpawnedLevelEntity>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if !keys.just_pressed(KeyCode::KeyL) || !ctrl || !shift {
        return;
    }

    toggle_hitbox_lines(&mut debug_settings);

    if debug_settings.show_hitbox_lines {
        for (target, health, damage, plasma, state) in &entity_query {
            let text = build_stats_text(health, damage, plasma, state);
            if text.is_empty() {
                continue;
            }
            commands.spawn((
                Text2d::new(text),
                crate::TextFont { font_size: 13.0, ..default() },
                crate::TextColor(Color::srgb(1.0, 1.0, 0.25)),
                Transform::default(),
                DebugStatsLabel { target },
                GameViewEntity,
            ));
        }
    } else {
        for entity in &label_query {
            commands.entity(entity).despawn();
        }
    }
}

