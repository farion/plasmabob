use bevy::prelude::*;

use crate::helper::input::{Action, ActionPressed};

/// Toggle FPS display update (used to turn off/on FPS sampling in the Debug HUD)
pub(crate) fn toggle_debug_fps(
    mut action_pressed: MessageReader<ActionPressed>,
    mut stats: ResMut<crate::game::debug_stats::DebugStats>,
) {
    for event in action_pressed.read() {
        if event.0 != Action::ToggleDebugFps {
            continue;
        }
        stats.show_fps = !stats.show_fps;
    }
}
