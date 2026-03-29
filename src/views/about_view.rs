use bevy::prelude::*;

use crate::AppState;

pub struct AboutViewPlugin;

#[derive(Component)]
struct AboutViewEntity;

impl Plugin for AboutViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AboutView), setup_about_view)
            .add_systems(
                Update,
                return_to_main_menu.run_if(in_state(AppState::AboutView)),
            )
            .add_systems(OnExit(AppState::AboutView), cleanup_about_view);
    }
}

fn setup_about_view(mut commands: Commands) {
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
            AboutViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("About"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                AboutViewEntity,
            ));
            parent.spawn((
                Text::new("Made proudly by Frieder Reinhold with Rust and Bevy."),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                AboutViewEntity,
            ));
            parent.spawn((
                Text::new("Press Esc to return"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                AboutViewEntity,
            ));
        });
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn cleanup_about_view(mut commands: Commands, entities: Query<Entity, (With<AboutViewEntity>, Without<Parent>)>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

