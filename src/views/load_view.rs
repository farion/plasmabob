use bevy::prelude::*;

use crate::level::CachedLevelDefinition;
use crate::{AppState, CampaignProgress, LevelSelection, PendingStoryScreen, StoryScreenRequest};

pub struct LoadViewPlugin;

#[derive(Component)]
struct LoadViewEntity;

impl Plugin for LoadViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadView),
            (load_level_for_game_view, setup_load_view).chain(),
        )
            .add_systems(
                Update,
                return_to_main_menu.run_if(in_state(AppState::LoadView)),
            )
            .add_systems(OnExit(AppState::LoadView), cleanup_load_view);
    }
}

fn load_level_for_game_view(
    asset_server: Res<AssetServer>,
    mut cached_level_definition: ResMut<CachedLevelDefinition>,
    level_selection: Res<LevelSelection>,
    mut pending_story: ResMut<PendingStoryScreen>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    cached_level_definition.refresh(&asset_server, level_selection.asset_path());

    if let Ok(level_definition) = cached_level_definition.level_definition() {
        if let Some(story) = level_definition
            .story
            .as_ref()
            .and_then(|story| story.start.as_ref())
        {
            pending_story.set(StoryScreenRequest {
                text_asset_path: story.text.clone(),
                background_asset_path: story.background.clone(),
                continue_to: AppState::GameView,
            });
            next_state.set(AppState::StoryView);
            return;
        }

        next_state.set(AppState::GameView);
    }
}

fn setup_load_view(
    mut commands: Commands,
    level_selection: Res<LevelSelection>,
    cached_level_definition: Res<CachedLevelDefinition>,
) {
    let (title, detail) = match cached_level_definition.level_definition() {
        Ok(_) => (
            "Load View".to_string(),
            format!("Loaded '{}'. Entering game...", level_selection.asset_path()),
        ),
        Err(error) => (
            "Could not load level".to_string(),
            format!("{}\nPress Esc to return", error),
        ),
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            LoadViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                LoadViewEntity,
            ));
            parent.spawn((
                Text::new(detail),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LoadViewEntity,
            ));
        });
}

fn return_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    progress: Res<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if progress.world_index.is_some() {
            next_state.set(AppState::WorldMapView);
        } else {
            next_state.set(AppState::MainMenu);
        }
    }
}

fn cleanup_load_view(mut commands: Commands, entities: Query<Entity, (With<LoadViewEntity>, Without<ChildOf>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

