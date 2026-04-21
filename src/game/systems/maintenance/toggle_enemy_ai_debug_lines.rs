use bevy::prelude::*;

/// Toggle enemy AI debug overlays with Ctrl+Shift+A.
pub(crate) fn toggle_enemy_ai_debug_lines(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl && shift && keys.just_pressed(KeyCode::KeyA) {
        debug_settings.show_enemy_ai_debug = !debug_settings.show_enemy_ai_debug;
    }
}
