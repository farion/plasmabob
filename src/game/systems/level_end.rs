use bevy::prelude::*;

use crate::app_model::AppState;
use crate::game::components::Health;
use crate::game::components::StateMachine;
use crate::game::tags::PlayerTag;
use crate::game::runtime_components::SpawnedLevelEntity;

/// Checks for level end conditions:
/// - player dead -> LoseView
/// - player overlaps an `exit` entity -> WinView
pub fn check_level_end(
    players: Query<(&Transform, Option<&Sprite>), With<PlayerTag>>,
    exits: Query<(
        &Transform,
        Option<&Sprite>,
        &SpawnedLevelEntity,
        Option<&StateMachine>,
    )>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Single-player: take the first player found.
    // bevy's Query method is `single()` (returns Result) in this version.
    let Ok((player_tf, player_sprite)) = players.single() else { return; };
    let player_pos = player_tf.translation;

    // If any exit entity overlaps the player, trigger WinView.
    for (exit_tf, exit_sprite_opt, spawned, state_machine) in &exits {
        if state_machine.is_some_and(|sm| sm.is_non_interactive()) {
            continue;
        }

        // Identify exit by entity_type or id (many levels use id "exit" or type "exit").
        if spawned.entity_type.to_ascii_lowercase() != "exit" && spawned.id.to_ascii_lowercase() != "exit" {
            continue;
        }

        // Determine bounding boxes. Use sprite.custom_size when available.
        let exit_pos = exit_tf.translation;
        let exit_half = exit_sprite_opt
            .and_then(|s| s.custom_size)
            .map(|v| v / 2.0)
            .unwrap_or(Vec2::splat(32.0));

        let player_half = player_sprite
            .and_then(|s| s.custom_size)
            .map(|v| v / 2.0)
            .unwrap_or(Vec2::splat(16.0));

        // Simple AABB overlap test in XY plane.
        let dx = (player_pos.x - exit_pos.x).abs();
        let dy = (player_pos.y - exit_pos.y).abs();

        if dx <= (player_half.x + exit_half.x) && dy <= (player_half.y + exit_half.y) {
            next_state.set(AppState::WinView);
            return;
        }
    }

    // Player-death is handled by `check_player_death` which runs separately.
}

pub fn check_player_death(
    healths: Query<&Health, With<PlayerTag>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for health in &healths {
        if health.is_dead() {
            next_state.set(AppState::LoseView);
            return;
        }
    }
}





