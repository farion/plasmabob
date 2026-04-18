use bevy::prelude::*;

use crate::app_model::AppState;
use crate::i18n::LocalizedText;
use crate::world::WorldCatalog;
use crate::{CampaignProgress, LevelSelection, LevelStats};

pub struct WinViewPlugin;

#[derive(Component)]
struct WinViewEntity;

impl Plugin for WinViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::WinView), setup_win_view)
            .add_systems(
                Update,
                (return_to_world_map, continue_campaign).run_if(in_state(AppState::WinView)),
            )
            .add_systems(OnExit(AppState::WinView), cleanup_win_view);
    }
}

fn setup_win_view(
    mut commands: Commands,
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    stats: Res<LevelStats>,
) {
    let has_next_level = next_level_json(&world_catalog, &progress).is_some();

    let title_key = if has_next_level {
        "win.level_cleared"
    } else {
        "win.planet_cleared"
    };
    let detail_key = if has_next_level {
        "win.detail_next"
    } else {
        "win.detail_done"
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
            WinViewEntity,
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
                    key: title_key.to_string(),
                },
                WinViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LocalizedText {
                    key: detail_key.to_string(),
                },
                WinViewEntity,
            ));

            // Statistics table
            parent
                .spawn((
                    Node {
                        width: Val::Px(480.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.06, 0.07)),
                    WinViewEntity,
                ))
                .with_children(|table| {
                    // rows: label (left) and value (right)
                    fn row(table: &mut ChildSpawnerCommands, label_key: &str, value: String) {
                        table
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                WinViewEntity,
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    LocalizedText {
                                        key: label_key.to_string(),
                                    },
                                    WinViewEntity,
                                ));
                                row.spawn((
                                    Text::new(value),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    WinViewEntity,
                                ));
                            });
                    }

                    let accuracy = if stats.shots == 0 {
                        0.0
                    } else {
                        (stats.enemies_killed as f32) / (stats.shots as f32)
                    };

                    row(
                        table,
                        "stats.enemies_killed",
                        format!("{}", stats.enemies_killed),
                    );
                    row(
                        table,
                        "stats.total_time",
                        format!("{:.2} s", stats.total_time_seconds),
                    );
                    row(table, "stats.jumps", format!("{}", stats.jumps));
                    row(table, "stats.shots", format!("{}", stats.shots));
                    row(table, "stats.accuracy", format!("{:.1}%", accuracy * 100.0));

                    table
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                margin: UiRect::top(Val::Px(8.0)),
                                ..default()
                            },
                            WinViewEntity,
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                LocalizedText {
                                    key: "stats.total_score".to_string(),
                                },
                                WinViewEntity,
                            ));
                            row.spawn((
                                Text::new(format!("{}", stats.score)),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.4)),
                                WinViewEntity,
                            ));
                        });
                });
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

fn continue_campaign(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    mut progress: ResMut<CampaignProgress>,
    mut level_selection: ResMut<LevelSelection>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        if let Some(level_path) = next_level_json(&world_catalog, &progress) {
            progress.level_index += 1;
            level_selection.set_asset_path(level_path);
            next_state.set(AppState::LoadView);
        } else {
            progress.clear_planet_progress();
            next_state.set(AppState::WorldMapView);
        }
    }
}

fn next_level_json<'a>(
    world_catalog: &'a WorldCatalog,
    progress: &CampaignProgress,
) -> Option<&'a str> {
    let world_index = progress.world_index?;
    let planet_index = progress.planet_index?;

    let world = &world_catalog.world(world_index)?.definition;
    let planet = world.planets.get(planet_index)?;
    let next_level_index = progress.level_index + 1;

    planet
        .levels
        .get(next_level_index)
        .map(|level| level.json.as_str())
}

fn cleanup_win_view(
    mut commands: Commands,
    entities: Query<Entity, (With<WinViewEntity>, Without<ChildOf>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
