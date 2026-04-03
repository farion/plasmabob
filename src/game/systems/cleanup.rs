use bevy::prelude::*;
use bevy::time::Virtual;

use crate::game::components::SpawnedLevelEntity;
use crate::MainCamera;

use super::{
    ActiveLevelBounds, CombatSoundEffects, GameViewEntity, LevelQuotes, PauseMenuState,
    QuoteCooldown,
};

pub(super) fn cleanup_game_view(
    mut commands: Commands,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut pause_menu_state: ResMut<PauseMenuState>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    ui_entities: Query<Entity, (With<GameViewEntity>, Without<ChildOf>)>,
    level_entities: Query<Entity, (With<SpawnedLevelEntity>, Without<ChildOf>)>,
) {
    commands.remove_resource::<ActiveLevelBounds>();
    commands.remove_resource::<LevelQuotes>();
    commands.remove_resource::<CombatSoundEffects>();
    commands.remove_resource::<QuoteCooldown>();

    virtual_time.unpause();
    pause_menu_state.is_open = false;
    pause_menu_state.selection = 0;
    pause_menu_state.suppress_enter_until_release = false;

    if let Some(mut camera_transform) = cameras.iter_mut().next() {
        camera_transform.translation.x = 0.0;
        camera_transform.translation.y = 0.0;
    }

    for entity in &ui_entities {
        commands.entity(entity).despawn();
    }

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }
}



