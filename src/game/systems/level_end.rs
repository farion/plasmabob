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
    cached_level: Option<Res<crate::game::level::types::CachedLevelDefinition>>,
    mut stats: ResMut<crate::LevelStats>,
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
            let (enemy_tag_count, environment_tag_count) = level_tag_counts(cached_level.as_deref());
            let elapsed_seconds = stats.total_time_seconds.max(1.0);
            let exit_bonus = (((5.0 * enemy_tag_count as f32) + (3.0 * environment_tag_count as f32))
                / elapsed_seconds)
                * 100.0;
            stats.exit_bonus = exit_bonus.max(0.0).round() as u64;
            stats.recompute_score();
            next_state.set(AppState::WinView);
            return;
        }
    }

    // Player-death is handled by `check_player_death` which runs separately.
}

fn level_tag_counts(
    cached_level: Option<&crate::game::level::types::CachedLevelDefinition>,
) -> (u32, u32) {
    let Some(cached_level) = cached_level else {
        return (0, 0);
    };
    let Some(level) = &cached_level.level else {
        return (0, 0);
    };
    let mut enemy_tags = 0_u32;
    let mut environment_tags = 0_u32;
    for entity in &level.entities {
        let Some(entity_type) = cached_level.entity_types.get(&entity.entity_type) else {
            continue;
        };
        let Some(category) = entity_type.category_tag.as_ref() else {
            continue;
        };
        match category.to_ascii_lowercase().as_str() {
            "enemy" => enemy_tags = enemy_tags.saturating_add(1),
            "environment" | "movingplatform" => {
                environment_tags = environment_tags.saturating_add(1)
            }
            _ => {}
        }
    }

    (enemy_tags, environment_tags)
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





