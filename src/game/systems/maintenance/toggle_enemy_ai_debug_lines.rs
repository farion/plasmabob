use bevy::prelude::*;

use crate::helper::input::{Action, ActionPressed};

/// Toggle enemy AI debug overlays with Alt+F4.
pub(crate) fn toggle_enemy_ai_debug_lines(
    mut action_pressed: MessageReader<ActionPressed>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
) {
    for event in action_pressed.read() {
        if event.0 != Action::ToggleEnemyAiDebug {
            continue;
        }
        debug_settings.show_enemy_ai_debug = !debug_settings.show_enemy_ai_debug;
    }
}
