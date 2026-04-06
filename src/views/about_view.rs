use bevy::prelude::*;

use crate::app_model::{AppState, StartScreenBackground};
use crate::helper::active_character::ActiveCharacter;
use crate::helper::i18n::LocalizedText;

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

fn setup_about_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_character: Res<ActiveCharacter>,
) {
    // Use the same background image as the main menu for visual consistency and character theme.
    commands.spawn((
        Sprite::from_image(asset_server.load(active_character.menu_background_path())),
        Transform::from_xyz(0.0, 0.0, -1.0),
        StartScreenBackground,
        AboutViewEntity,
    ));

    // Make the root UI cover the whole viewport so the view scales to the window
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            AboutViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                LocalizedText {
                    key: "about.title".to_string(),
                },
                AboutViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                LocalizedText {
                    key: "about.blurb".to_string(),
                },
                AboutViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LocalizedText {
                    key: "about.hint".to_string(),
                },
                AboutViewEntity,
            ));
        });
}

fn return_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn cleanup_about_view(
    mut commands: Commands,
    entities: Query<Entity, (With<AboutViewEntity>, Without<ChildOf>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
