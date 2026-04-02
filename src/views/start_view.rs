use bevy::prelude::*;

use crate::game::world::WorldCatalog;
use crate::{AppState, CampaignProgress, WorldListSelection};

pub struct StartViewPlugin;

#[derive(Component)]
struct StartViewEntity;

#[derive(Component)]
struct WorldListItem {
    index: usize,
}

impl Plugin for StartViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::StartView),
            (refresh_world_catalog, setup_start_view).chain(),
        )
            .add_systems(
                Update,
                (
                    world_list_keyboard_navigation,
                    activate_selected_world,
                    return_to_main_menu,
                    update_world_list_visuals,
                )
                    .run_if(in_state(AppState::StartView)),
            )
            .add_systems(OnExit(AppState::StartView), cleanup_start_view);
    }
}

fn refresh_world_catalog(
    asset_server: Res<AssetServer>,
    mut world_catalog: ResMut<WorldCatalog>,
    mut selection: ResMut<WorldListSelection>,
    mut progress: ResMut<CampaignProgress>,
) {
    world_catalog.refresh(&asset_server);
    selection.index = 0;
    progress.clear_planet_progress();
}

fn setup_start_view(mut commands: Commands, world_catalog: Res<WorldCatalog>) {
    let (title, subtitle) = if world_catalog.worlds().is_empty() {
        let detail = world_catalog
            .last_error()
            .unwrap_or("Keine Welt-JSONs in assets/worlds gefunden.");
        (
            "Weltenauswahl nicht verfuegbar".to_string(),
            format!("{detail}\nEsc: Zurueck zum Hauptmenue"),
        )
    } else {
        (
            "Waehle eine Welt".to_string(),
            "Pfeiltasten: Navigation | Enter: Weltkarte | Esc: Zurueck".to_string(),
        )
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
            StartViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StartViewEntity,
            ));
            parent.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                StartViewEntity,
            ));

            for (index, world) in world_catalog.worlds().iter().enumerate() {
                parent.spawn((
                    Text::new(format!("{} ({})", world.definition.name, world.asset_path)),
                    TextFont {
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    WorldListItem { index },
                    StartViewEntity,
                ));
            }
        });
}

fn world_list_keyboard_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    mut selection: ResMut<WorldListSelection>,
) {
    let count = world_catalog.worlds().len();
    if count == 0 {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        selection.index = (selection.index + 1) % count;
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        selection.index = if selection.index == 0 {
            count - 1
        } else {
            selection.index - 1
        };
    }
}

fn activate_selected_world(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    selection: Res<WorldListSelection>,
    mut progress: ResMut<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if world_catalog.worlds().is_empty() {
        return;
    }

    if !keys.just_pressed(KeyCode::Enter) && !keys.just_pressed(KeyCode::NumpadEnter) {
        return;
    }

    progress.world_index = Some(selection.index);
    progress.clear_planet_progress();
    next_state.set(AppState::WorldMapView);
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn update_world_list_visuals(
    selection: Res<WorldListSelection>,
    mut worlds: Query<(&WorldListItem, &mut TextColor)>,
) {
    for (item, mut color) in &mut worlds {
        *color = if item.index == selection.index {
            TextColor(Color::srgb(0.3, 0.6, 1.0))
        } else {
            TextColor(Color::WHITE)
        };
    }
}

fn cleanup_start_view(mut commands: Commands, entities: Query<Entity, (With<StartViewEntity>, Without<Parent>)>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
