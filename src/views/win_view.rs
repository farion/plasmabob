use bevy::prelude::*;

use crate::AppState;

pub struct WinViewPlugin;

#[derive(Component)]
struct WinViewEntity;

impl Plugin for WinViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::WinView), setup_win_view)
            .add_systems(
                Update,
                (return_to_main_menu, restart_level).run_if(in_state(AppState::WinView)),
            )
            .add_systems(OnExit(AppState::WinView), cleanup_win_view);
    }
}

fn setup_win_view(mut commands: Commands) {
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
            WinViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("You Win!"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WinViewEntity,
            ));
            parent.spawn((
                Text::new("Press Enter to play again"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                WinViewEntity,
            ));
            parent.spawn((
                Text::new("Press Esc to return"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                WinViewEntity,
            ));
        });
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn restart_level(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        next_state.set(AppState::GameView);
    }
}

fn cleanup_win_view(mut commands: Commands, entities: Query<Entity, With<WinViewEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

