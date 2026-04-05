use bevy::prelude::*;

use crate::AppState;
use crate::{PendingStoryScreen, StoryScreenRequest};
use crate::level::CachedLevelDefinition;

pub(crate) fn detect_player_defeated(
    player_query: Query<&crate::game::components::health::Health, With<crate::game::components::player::Player>>,
    cached_level_definition: Option<Res<CachedLevelDefinition>>,
    mut pending_story: Option<ResMut<PendingStoryScreen>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for health in &player_query {
        if health.is_dead() {
            info!("Player defeated - showing lose view.");
            if let (Some(level), Some(pending_story)) = (cached_level_definition.as_ref(), pending_story.as_mut()) {
                if let Ok(level_definition) = level.level_definition() {
                    if let Some(story) = level_definition
                        .story
                        .as_ref()
                        .and_then(|story| story.lose.as_ref())
                    {
                        pending_story.set(StoryScreenRequest {
                            text_asset_path: story.text.clone(),
                            background_asset_path: story.background.clone(),
                            continue_to: AppState::LoseView,
                        });
                        next_state.set(AppState::StoryView);
                        return;
                    }
                }
            }

            next_state.set(AppState::LoseView);
            return;
        }
    }
}

