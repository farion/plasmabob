use bevy::prelude::*;

use super::draw_hitbox_debug_lines::DebugHitbox;
use super::update_debug_stats_labels::DebugStatsLabel;
use crate::helper::input::{Action, ActionPressed};

/// Toggle `DebugRenderSettings.show_hitbox_lines` with Alt+F5. When
/// disabling the debug view we also despawn all debug helper entities.
pub(crate) fn toggle_hitbox_debug_lines(
    mut commands: Commands,
    mut action_pressed: MessageReader<ActionPressed>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
    hitbox_query: Query<Entity, With<DebugHitbox>>,
    label_query: Query<Entity, With<DebugStatsLabel>>,
) {
    for event in action_pressed.read() {
        if event.0 != Action::ToggleHitboxDebug {
            continue;
        }
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
