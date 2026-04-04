use bevy::prelude::*;

use crate::game::world::WorldCatalog;
use crate::{AppState, CampaignProgress, WorldListSelection};
use crate::i18n::{Translations, CurrentLanguage};

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

fn setup_start_view(
    mut commands: Commands,
    world_catalog: Res<WorldCatalog>,
    translations: Res<Translations>,
    current: Res<CurrentLanguage>,
) {
    // Determine title/subtitle keys or formatted detail when no worlds are available
        let (title_key, subtitle_text) = if world_catalog.worlds().is_empty() {
        let detail = world_catalog
            .last_error()
            .unwrap_or("No world JSONs found in assets/worlds.");
        // Use localized template and insert detail
            let template = translations
                .tr(&current.effective(&translations), "start.no_worlds_detail")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "{detail}\nEsc: Back to main menu".to_string());
        let subtitle = template.replace("{detail}", detail);
        ("start.title_no_worlds", subtitle)
    } else {
        (
            "start.title",
                translations
                    .tr(&current.effective(&translations), "start.subtitle")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Arrow keys: navigate | Enter: world map | Esc: back".to_string()),
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
                Text::new(""),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                crate::i18n::LocalizedText { key: title_key.to_string() },
                StartViewEntity,
            ));
            parent.spawn((
                // subtitle may contain a formatted detail when no worlds exist, so we
                // insert the prepared text directly rather than a LocalizedText key.
                Text::new(subtitle_text),
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
    progress.world_start_story_seen = false;
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

fn cleanup_start_view(mut commands: Commands, entities: Query<Entity, (With<StartViewEntity>, Without<ChildOf>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
