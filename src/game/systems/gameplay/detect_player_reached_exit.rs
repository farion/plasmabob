use bevy::prelude::*;

use crate::app_model::AppState;
use crate::game::components::exit::Exit;
use crate::{PendingStoryScreen, StoryScreenRequest};
use crate::level::CachedLevelDefinition;

pub(crate) fn detect_player_reached_exit(
    player_query: Query<(&avian2d::prelude::CollidingEntities, &crate::game::components::health::Health), With<crate::game::components::player::Player>>,
    exit_query: Query<(), With<Exit>>,
    cached_level_definition: Option<Res<CachedLevelDefinition>>,
    mut pending_story: Option<ResMut<PendingStoryScreen>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (colliding_entities, health) in &player_query {
        if health.is_dead() { continue; }

        if colliding_entities.0.iter().any(|entity| exit_query.contains(*entity)) {
            info!("Player reached exit - level won.");
            if let (Some(level), Some(pending_story)) = (cached_level_definition.as_ref(), pending_story.as_mut()) {
                if let Ok(level_definition) = level.level_definition() {
                    if let Some(story) = level_definition.story.as_ref().and_then(|story| story.win.as_ref()) {
                        pending_story.set(StoryScreenRequest {
                            text_asset_path: story.text.clone(),
                            background_asset_path: story.background.clone(),
                            continue_to: AppState::WinView,
                        });
                        next_state.set(AppState::StoryView);
                        return;
                    }
                }
            }

            next_state.set(AppState::WinView);
            return;
        }
    }
}

