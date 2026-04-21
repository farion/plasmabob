use bevy::prelude::*;

use super::draw_hitbox_debug_lines::DebugHitbox;
use super::update_debug_stats_labels::DebugStatsLabel;

/// Toggle `DebugRenderSettings.show_hitbox_lines` with Ctrl+Shift+L. When
/// disabling the debug view we also despawn all debug helper entities.
pub(crate) fn toggle_hitbox_debug_lines(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
    hitbox_query: Query<Entity, With<DebugHitbox>>,
    label_query: Query<Entity, With<DebugStatsLabel>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl && shift && keys.just_pressed(KeyCode::KeyL) {
        debug_settings.show_hitbox_lines = !debug_settings.show_hitbox_lines;

        if !debug_settings.show_hitbox_lines {
            // Despawn any debug helper entities.
            for e in &hitbox_query {
                commands.entity(e).try_despawn();
            }
            for e in &label_query {
                commands.entity(e).try_despawn();
            }
        }
    }
}
