use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::MainCamera;
use crate::game::components::player::Player;
use crate::game::systems::presentation::helpers::update_camera_x;
use crate::game::systems::systems_api::ActiveLevelBounds;

pub(crate) fn snap_camera_to_player(
    windows: Query<&Window, With<PrimaryWindow>>,
    active_level_bounds: Option<Res<ActiveLevelBounds>>,
    players: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = players.single() else {
        return;
    };
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };
    update_camera_x(
        &mut camera_transform,
        player_transform.translation.x,
        window.width(),
        active_level_bounds.as_deref().copied(),
    );
}
