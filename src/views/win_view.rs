use bevy::prelude::*;

use crate::game::world::WorldCatalog;
use crate::{AppState, CampaignProgress, LevelSelection};

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
) {
    let has_next_level = next_level_json(&world_catalog, &progress).is_some();

    let title = if has_next_level {
        "Level geschafft!"
    } else {
        "Planet abgeschlossen!"
    };

    let detail = if has_next_level {
        "Enter: Naechstes Level | Esc: Weltkarte"
    } else {
        "Enter: Zurueck zur Weltkarte | Esc: Weltkarte"
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
                Text::new(title),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WinViewEntity,
            ));
            parent.spawn((
                Text::new(detail),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                WinViewEntity,
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

    planet.levels.get(next_level_index).map(|level| level.json.as_str())
}

fn cleanup_win_view(mut commands: Commands, entities: Query<Entity, (With<WinViewEntity>, Without<Parent>)>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

