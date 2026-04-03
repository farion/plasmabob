use bevy::prelude::*;

use crate::{AppState, CampaignProgress};
use crate::i18n::LocalizedText;

pub struct LoseViewPlugin;

#[derive(Component)]
struct LoseViewEntity;

impl Plugin for LoseViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoseView), setup_lose_view)
            .add_systems(
                Update,
                (return_to_world_map, restart_level).run_if(in_state(AppState::LoseView)),
            )
            .add_systems(OnExit(AppState::LoseView), cleanup_lose_view);
    }
}

fn setup_lose_view(mut commands: Commands) {
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
            LoseViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                LocalizedText { key: "lose.title".to_string() },
                LoseViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LocalizedText { key: "lose.repeat".to_string() },
                LoseViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                LocalizedText { key: "lose.cancel".to_string() },
                LoseViewEntity,
            ));
        });
}

fn return_to_world_map(
    keys: Res<ButtonInput<KeyCode>>,
    mut progress: ResMut<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        progress.clear_planet_progress();
        next_state.set(AppState::WorldMapView);
    }
}

fn restart_level(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        next_state.set(AppState::LoadView);
    }
}

fn cleanup_lose_view(mut commands: Commands, entities: Query<Entity, (With<LoseViewEntity>, Without<ChildOf>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

