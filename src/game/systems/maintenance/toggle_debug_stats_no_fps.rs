use bevy::prelude::*;

use crate::helper::input::{Action, ActionPressed};

/// Toggle DebugStats visibility without FPS (Alt+F3 handled in HUD plugin for visibility).
/// This system clears the FPS value when toggled to 'without FPS'.
pub(crate) fn toggle_debug_stats_no_fps(
    mut action_pressed: MessageReader<ActionPressed>,
    mut stats: ResMut<crate::game::debug_stats::DebugStats>,
) {
    for event in action_pressed.read() {
        if event.0 != Action::ToggleDebugCounters {
            continue;
        }
        stats.show_counters = !stats.show_counters;
    }
}
