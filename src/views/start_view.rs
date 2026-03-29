use bevy::prelude::*;

use crate::AppState;

pub struct StartViewPlugin;

#[derive(Component)]
struct StartViewEntity;

impl Plugin for StartViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartView), setup_start_view)
            .add_systems(
                Update,
                (start_game, return_to_main_menu).run_if(in_state(AppState::StartView)),
            )
            .add_systems(OnExit(AppState::StartView), cleanup_start_view);
    }
}

fn setup_start_view(mut commands: Commands) {
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
            StartViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PlasmaBob Level 1 - Get Ready!"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StartViewEntity,
            ));
            parent.spawn((
                Text::new("Control movement with Arrow left and right. Jump with Arrow Up. Shot your plasma gun with space."),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                StartViewEntity,
            ));
            parent.spawn((
                Text::new("Press Enter to start"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                StartViewEntity,
            ));
            parent.spawn((
                Text::new("Press Esc to return"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                StartViewEntity,
            ));
        });
}

fn start_game(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        next_state.set(AppState::GameView);
    }
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn cleanup_start_view(mut commands: Commands, entities: Query<Entity, (With<StartViewEntity>, Without<Parent>)>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
