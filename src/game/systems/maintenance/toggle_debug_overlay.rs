use bevy::prelude::*;

pub(crate) fn toggle_debug_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<crate::DebugRenderSettings>,
    mut overlay: Query<&mut Visibility, With<crate::game::systems::systems_api::DebugOverlayRoot>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl && shift && keys.just_pressed(KeyCode::KeyO) {
        debug_settings.show_overlay = !debug_settings.show_overlay;
        let visibility = if debug_settings.show_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        for mut vis in &mut overlay {
            *vis = visibility;
        }
    }
}

