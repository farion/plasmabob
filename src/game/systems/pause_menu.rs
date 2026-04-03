use bevy::prelude::*;
use bevy::time::Virtual;
use bevy::ui::FocusPolicy;

use crate::{AppState, CampaignProgress};

use super::{GameViewEntity, PauseMenuAction, PauseMenuState, PAUSE_MENU_ITEMS};

#[derive(Component)]
pub(super) struct PauseMenuRoot;

#[derive(Component)]
pub(super) struct PauseMenuButton {
    index: usize,
    action: PauseMenuAction,
}

pub(super) fn update_pause_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    interactions: Query<(&Interaction, &PauseMenuButton), (Changed<Interaction>, With<Button>)>,
    mut button_query: Query<(&PauseMenuButton, &Children, &mut BackgroundColor), With<Button>>,
    mut text_colors: Query<&mut TextColor>,
    roots: Query<Entity, With<PauseMenuRoot>>,
    mut pause_menu_state: ResMut<PauseMenuState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut progress: ResMut<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if pause_menu_state.is_open {
            pause_menu_state.is_open = false;
            pause_menu_state.selection = PAUSE_MENU_ITEMS.len().saturating_sub(1);
        } else {
            pause_menu_state.is_open = true;
            pause_menu_state.selection = PAUSE_MENU_ITEMS.len().saturating_sub(1);
        }
    }

    if !pause_menu_state.is_open {
        if pause_menu_state.suppress_enter_until_release && !is_enter_pressed(&keys) {
            pause_menu_state.suppress_enter_until_release = false;
        }

        virtual_time.unpause();
        for entity in &roots {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    virtual_time.pause();

    if roots.iter().next().is_none() {
        spawn_pause_menu(&mut commands);
    }

    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowLeft) {
        pause_menu_state.selection = if pause_menu_state.selection == 0 {
            PAUSE_MENU_ITEMS.len() - 1
        } else {
            pause_menu_state.selection - 1
        };
    }

    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowRight) {
        pause_menu_state.selection = (pause_menu_state.selection + 1) % PAUSE_MENU_ITEMS.len();
    }

    for (interaction, button) in &interactions {
        match *interaction {
            Interaction::Hovered => {
                pause_menu_state.selection = button.index;
            }
            Interaction::Pressed => {
                pause_menu_state.selection = button.index;
                execute_action(
                    button.action,
                    &mut pause_menu_state,
                    &mut virtual_time,
                    &mut progress,
                    &mut next_state,
                );
            }
            Interaction::None => {}
        }
    }

    if pause_menu_state.suppress_enter_until_release {
        if !is_enter_pressed(&keys) {
            pause_menu_state.suppress_enter_until_release = false;
        }
    } else if is_enter_just_pressed(&keys) {
        pause_menu_state.suppress_enter_until_release = true;
        let (_, action) = PAUSE_MENU_ITEMS[pause_menu_state.selection];
        execute_action(
            action,
            &mut pause_menu_state,
            &mut virtual_time,
            &mut progress,
            &mut next_state,
        );
    }

    for (button, children, mut background) in &mut button_query {
        let is_selected = button.index == pause_menu_state.selection;
        *background = BackgroundColor(Color::NONE);

        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(*child) {
                *text_color = if is_selected {
                    TextColor(Color::srgb(0.3, 0.6, 1.0))
                } else {
                    TextColor(Color::WHITE)
                };
            }
        }
    }
}

fn is_enter_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::Enter) || keys.pressed(KeyCode::NumpadEnter)
}

fn is_enter_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)
}

fn spawn_pause_menu(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            FocusPolicy::Block,
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuRoot,
            GameViewEntity,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: Val::Px(620.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.06, 0.08)),
                    GameViewEntity,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        crate::i18n::LocalizedText { key: "pause.title".to_string() },
                        GameViewEntity,
                    ));
                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 21.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 0.72, 0.72)),
                        crate::i18n::LocalizedText { key: "pause.hint".to_string() },
                        GameViewEntity,
                    ));

                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            GameViewEntity,
                        ))
                        .with_children(|button_list| {
                            for (index, (label, action)) in PAUSE_MENU_ITEMS.into_iter().enumerate() {
                                button_list
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        PauseMenuButton { index, action },
                                        GameViewEntity,
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(""),
                                            TextFont {
                                                font_size: 34.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                            crate::i18n::LocalizedText { key: label.to_string() },
                                            GameViewEntity,
                                        ));
                                    });
                            }
                        });
                });
        });
}

fn execute_action(
    action: PauseMenuAction,
    pause_menu_state: &mut ResMut<PauseMenuState>,
    virtual_time: &mut ResMut<Time<Virtual>>,
    progress: &mut ResMut<CampaignProgress>,
    next_state: &mut ResMut<NextState<AppState>>,
) {
    match action {
        PauseMenuAction::Restart => {
            pause_menu_state.is_open = false;
            virtual_time.unpause();
            next_state.set(AppState::LoadView);
        }
        PauseMenuAction::BackToWorldMap => {
            pause_menu_state.is_open = false;
            progress.clear_planet_progress();
            virtual_time.unpause();
            next_state.set(AppState::WorldMapView);
        }
        PauseMenuAction::BackToMainMenu => {
            pause_menu_state.is_open = false;
            progress.world_index = None;
            progress.clear_planet_progress();
            progress.world_start_story_seen = false;
            virtual_time.unpause();
            next_state.set(AppState::MainMenu);
        }
        PauseMenuAction::Cancel => {
            pause_menu_state.is_open = false;
            pause_menu_state.selection = PAUSE_MENU_ITEMS.len().saturating_sub(1);
            virtual_time.unpause();
        }
    }
}




