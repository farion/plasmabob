use bevy::prelude::*;
/// Toggle parallax rendering with Ctrl+Shift+P.
pub fn toggle_parallax_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl && shift && keys.just_pressed(KeyCode::KeyP) {
        debug_settings.parallax_enabled = !debug_settings.parallax_enabled;
    }
}
