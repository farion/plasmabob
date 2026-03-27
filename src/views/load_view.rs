use bevy::prelude::*;

use crate::AppState;

pub struct LoadViewPlugin;

#[derive(Component)]
struct LoadViewEntity;

impl Plugin for LoadViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadView), setup_load_view)
            .add_systems(
                Update,
                return_to_main_menu.run_if(in_state(AppState::LoadView)),
            )
            .add_systems(OnExit(AppState::LoadView), cleanup_load_view);
    }
}

fn setup_load_view(mut commands: Commands) {
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
                Text::new("Load View"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                LoadViewEntity,
            ));
            parent.spawn((
                Text::new("Press Esc to return"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LoadViewEntity,
            ));
        });
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn cleanup_load_view(mut commands: Commands, entities: Query<Entity, With<LoadViewEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

