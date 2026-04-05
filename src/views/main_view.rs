use bevy::prelude::*;
use bevy::ui::ZIndex;
use bevy::window::PrimaryWindow;

use crate::app_model::{
    AppState, ExitConfirmAction, ExitConfirmButton, ExitConfirmModalRoot, ExitConfirmModalState,
    MainMenuEntity, MenuAction, MenuButton, MenuMusicEntity, MenuSelection, StartScreenBackground,
    EXIT_CONFIRM_ITEMS, MENU_ITEMS,
};
use crate::helper::i18n;

pub struct MainViewPlugin;

impl Plugin for MainViewPlugin {
    fn build(&self, app: &mut App) {
        // Define ordered SystemSets for the main menu so Input -> Action -> Visual is explicit.
        #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
        pub enum MainMenuSet {
            Input,
            Modal,
            Action,
            Visual,
        }

        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnEnter(AppState::StartView), stop_menu_music)
            .add_systems(OnExit(AppState::MainMenu), stop_menu_music_on_main_exit)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
            // Keep background sprites fitted for all views that use StartScreenBackground.
            .add_systems(Update, fit_background_to_window)
            // Configure the ordering of our MainMenu system sets on Update
            // Order: Input -> Modal -> Action -> Visual
            .configure_sets(
                Update,
                (
                    MainMenuSet::Input,
                    MainMenuSet::Modal,
                    MainMenuSet::Action,
                    MainMenuSet::Visual,
                ),
            )
            // Input: collect global user input and pointer events (menu-level)
            .add_systems(
                Update,
                (
                    open_or_close_exit_modal_with_escape,
                    menu_keyboard_navigation,
                    menu_pointer_input,
                )
                    .in_set(MainMenuSet::Input)
                    .run_if(in_state(AppState::MainMenu)),
            )
            // Modal: input specific to the exit-confirm modal
            .add_systems(
                Update,
                (
                    modal_keyboard_navigation,
                    modal_pointer_input,
                )
                    .in_set(MainMenuSet::Modal)
                    .run_if(in_state(AppState::MainMenu)),
            )
            // Action: activation systems that perform state changes
            .add_systems(
                Update,
                (
                    activate_selected_menu_item,
                    activate_selected_modal_item,
                )
                    .in_set(MainMenuSet::Action)
                    .run_if(in_state(AppState::MainMenu)),
            )
            // Visual: update visuals and modal syncing after actions
            .add_systems(
                Update,
                (
                    update_menu_visuals,
                    update_modal_visuals,
                    sync_exit_modal,
                )
                    .in_set(MainMenuSet::Visual)
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

/// Spawn the main menu UI entities (logo, sidebar, menu buttons, footer).
/// This function is called from `setup_main_menu` in `main.rs`.
pub fn spawn_main_menu_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    // Background image
    commands.spawn((
        Sprite::from_image(asset_server.load("start.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
        MainMenuEntity,
        StartScreenBackground,
    ));

    // Logo
    commands.spawn((
        Node {
            width: Val::Px(400.0),
            height: Val::Auto,
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            left: Val::Px(50.0),
            ..default()
        },
        ImageNode::new(asset_server.load("logo.png")),
        ZIndex(200),
        MainMenuEntity,
    ));

    // Sidebar with menu buttons
    commands
        .spawn((
            Node {
                width: Val::Px(512.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(32.0), Val::Px(24.0)),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(100),
            MainMenuEntity,
        ))
        .with_children(|parent| {
            for (index, (label, action)) in MENU_ITEMS.into_iter().enumerate() {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        MenuButton { index, action },
                        MainMenuEntity,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(""),
                            TextFont { font_size: 46.0, ..default() },
                            TextColor(Color::WHITE),
                            i18n::LocalizedText { key: label.to_string() },
                            MainMenuEntity,
                        ));
                    });
            }
        });

    // Footer text
    commands.spawn((
        Node {
            width: Val::Auto,
            height: Val::Auto,
            position_type: PositionType::Absolute,
            right: Val::Percent(5.0),
            bottom: Val::Percent(5.0),
            ..default()
        },
        Text::new("Beinhaltet Sarkasmus und Klischees"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.72, 0.72, 0.72)),
        MainMenuEntity,
    ));
}

/// System: setup the main menu state. Initializes selection/modal state, refreshes
/// world catalog and ensures menu music is playing, then spawns the UI.
pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<crate::helper::audio_settings::AudioSettings>,
    mut selection: ResMut<MenuSelection>,
    mut modal_state: ResMut<ExitConfirmModalState>,
    mut world_catalog: ResMut<crate::world::WorldCatalog>,
    menu_music_entities: Query<Entity, With<MenuMusicEntity>>,
) {
    selection.index = 0;
    modal_state.is_open = false;
    modal_state.selection = 1;
    modal_state.suppress_enter_until_release = false;

    // Refresh world catalog early so the main menu can make decisions.
    world_catalog.refresh(&asset_server);

    if menu_music_entities.iter().next().is_none() {
        commands.spawn((
            bevy::audio::AudioPlayer::new(asset_server.load("music/start.ogg")),
            bevy::audio::PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Linear(audio_settings.music_volume),
                ..default()
            },
            MenuMusicEntity,
        ));
    }

    // Spawn the visual UI (logo, sidebar, footer)
    spawn_main_menu_ui(&mut commands, &asset_server);
}

pub(crate) fn cleanup_main_menu(
    mut commands: Commands,
    entities: Query<Entity, (With<MainMenuEntity>, Without<ChildOf>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn stop_menu_music(
    mut commands: Commands,
    music_entities: Query<Entity, With<MenuMusicEntity>>,
) {
    for entity in &music_entities {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn stop_menu_music_on_main_exit(
    mut commands: Commands,
    music_entities: Query<Entity, With<MenuMusicEntity>>,
    next_state: Option<Res<NextState<AppState>>>,
) {
    let should_stop = match next_state {
        Some(ns) => match &*ns {
            NextState::Pending(AppState::SettingsView)
            | NextState::Pending(AppState::AboutView) => false,
            _ => true,
        },
        None => true,
    };

    if !should_stop {
        return;
    }

    for entity in &music_entities {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn menu_keyboard_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MenuSelection>,
    modal_state: Res<ExitConfirmModalState>,
) {
    if modal_state.is_open {
        return;
    }

    if selection.index >= MENU_ITEMS.len() {
        selection.index = 0;
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        selection.index = (selection.index + 1) % MENU_ITEMS.len();
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        if selection.index == 0 {
            selection.index = MENU_ITEMS.len() - 1;
        } else {
            selection.index -= 1;
        }
    }
}

pub(crate) fn menu_pointer_input(
    interactions: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut selection: ResMut<MenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit: MessageWriter<AppExit>,
    world_catalog: Res<crate::world::WorldCatalog>,
    mut progress: ResMut<crate::CampaignProgress>,
    mut modal_state: ResMut<ExitConfirmModalState>,
) {
    if modal_state.is_open {
        return;
    }

    for (interaction, button) in &interactions {
        match *interaction {
            Interaction::Hovered => {
                selection.index = button.index;
            }
            Interaction::Pressed => {
                selection.index = button.index;
                activate_action(
                    button.action,
                    &mut next_state,
                    &mut app_exit,
                    &world_catalog,
                    &mut progress,
                    &mut modal_state,
                );
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn activate_selected_menu_item(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit: MessageWriter<AppExit>,
    world_catalog: Res<crate::world::WorldCatalog>,
    mut progress: ResMut<crate::CampaignProgress>,
    mut modal_state: ResMut<ExitConfirmModalState>,
) {
    if modal_state.is_open {
        return;
    }

    if modal_state.suppress_enter_until_release {
        if !is_enter_pressed(&keys) {
            modal_state.suppress_enter_until_release = false;
        }
        return;
    }

    if !is_enter_just_pressed(&keys) {
        return;
    }

    let (_, action) = MENU_ITEMS[selection.index];
    activate_action(
        action,
        &mut next_state,
        &mut app_exit,
        &world_catalog,
        &mut progress,
        &mut modal_state,
    );
}

fn activate_action(
    action: MenuAction,
    next_state: &mut ResMut<NextState<AppState>>,
    app_exit: &mut MessageWriter<AppExit>,
    world_catalog: &Res<crate::world::WorldCatalog>,
    progress: &mut ResMut<crate::CampaignProgress>,
    modal_state: &mut ResMut<ExitConfirmModalState>,
) {
    match action {
        MenuAction::Start => {
            let count = world_catalog.worlds().len();
            if count == 1 {
                progress.world_index = Some(0);
                progress.clear_planet_progress();
                progress.world_start_story_seen = false;
                next_state.set(AppState::WorldMapView);
            } else {
                next_state.set(AppState::StartView);
            }
        }
        MenuAction::Settings => next_state.set(AppState::SettingsView),
        MenuAction::About => next_state.set(AppState::AboutView),
        MenuAction::Exit => {
            let _ = app_exit;
            modal_state.is_open = true;
            modal_state.selection = 1;
            modal_state.suppress_enter_until_release = true;
        }
    }
}

pub(crate) fn open_or_close_exit_modal_with_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut modal_state: ResMut<ExitConfirmModalState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if modal_state.is_open {
        modal_state.is_open = false;
    } else {
        modal_state.is_open = true;
        modal_state.selection = 1;
    }
}

pub(crate) fn modal_keyboard_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut modal_state: ResMut<ExitConfirmModalState>,
) {
    if !modal_state.is_open {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowUp) {
        modal_state.selection = if modal_state.selection == 0 {
            EXIT_CONFIRM_ITEMS.len() - 1
        } else {
            modal_state.selection - 1
        };
    }

    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::ArrowDown) {
        modal_state.selection = (modal_state.selection + 1) % EXIT_CONFIRM_ITEMS.len();
    }
}

pub(crate) fn modal_pointer_input(
    interactions: Query<(&Interaction, &ExitConfirmButton), (Changed<Interaction>, With<Button>)>,
    mut modal_state: ResMut<ExitConfirmModalState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !modal_state.is_open {
        return;
    }

    for (interaction, button) in &interactions {
        match *interaction {
            Interaction::Hovered => {
                modal_state.selection = button.index;
            }
            Interaction::Pressed => {
                modal_state.selection = button.index;
                execute_exit_modal_action(button.action, &mut modal_state, &mut app_exit);
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn activate_selected_modal_item(
    keys: Res<ButtonInput<KeyCode>>,
    mut modal_state: ResMut<ExitConfirmModalState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !modal_state.is_open {
        return;
    }

    if modal_state.suppress_enter_until_release {
        if !is_enter_pressed(&keys) {
            modal_state.suppress_enter_until_release = false;
        }
        return;
    }

    if !is_enter_just_pressed(&keys) {
        return;
    }

    modal_state.suppress_enter_until_release = true;

    let (_, action) = EXIT_CONFIRM_ITEMS[modal_state.selection];
    execute_exit_modal_action(action, &mut modal_state, &mut app_exit);
}

fn is_enter_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::Enter) || keys.pressed(KeyCode::NumpadEnter)
}

fn is_enter_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)
}

fn execute_exit_modal_action(
    action: ExitConfirmAction,
    modal_state: &mut ResMut<ExitConfirmModalState>,
    app_exit: &mut MessageWriter<AppExit>,
) {
    match action {
        ExitConfirmAction::Confirm => {
            app_exit.write(AppExit::Success);
        }
        ExitConfirmAction::Cancel => {
            modal_state.is_open = false;
            modal_state.selection = 1;
        }
    }
}

pub(crate) fn sync_exit_modal(
    mut commands: Commands,
    modal_state: Res<ExitConfirmModalState>,
    roots: Query<Entity, With<ExitConfirmModalRoot>>,
) {
    if modal_state.is_open {
        if roots.iter().next().is_some() {
            return;
        }

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
                bevy::ui::FocusPolicy::Block,
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ExitConfirmModalRoot,
                MainMenuEntity,
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Px(520.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.06, 0.06, 0.08)),
                        MainMenuEntity,
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 34.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            i18n::LocalizedText {
                                key: "modal.exit.title".to_string(),
                            },
                            MainMenuEntity,
                        ));
                        panel.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.72, 0.72, 0.72)),
                            i18n::LocalizedText {
                                key: "modal.exit.hint".to_string(),
                            },
                            MainMenuEntity,
                        ));

                        panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(12.0),
                                    ..default()
                                },
                                MainMenuEntity,
                            ))
                            .with_children(|button_list| {
                                for (index, (label, action)) in EXIT_CONFIRM_ITEMS.into_iter().enumerate() {
                                    button_list
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                padding: UiRect::all(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                            ExitConfirmButton { index, action },
                                            MainMenuEntity,
                                        ))
                                        .with_children(|button| {
                                            button.spawn((
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 34.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                                i18n::LocalizedText {
                                                    key: label.to_string(),
                                                },
                                                MainMenuEntity,
                                            ));
                                        });
                                }
                            });
                    });
            });
        return;
    }

    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn update_modal_visuals(
    modal_state: Res<ExitConfirmModalState>,
    mut button_query: Query<
        (&ExitConfirmButton, &Children, &mut BackgroundColor),
        With<Button>,
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    if !modal_state.is_open {
        return;
    }

    for (button, children, mut background) in &mut button_query {
        let is_selected = button.index == modal_state.selection;
        *background = BackgroundColor(Color::NONE);

        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                *text_color = if is_selected {
                    TextColor(Color::srgb(0.3, 0.6, 1.0))
                } else {
                    TextColor(Color::WHITE)
                };
            }
        }
    }
}

pub(crate) fn update_menu_visuals(
    selection: Res<MenuSelection>,
    modal_state: Res<ExitConfirmModalState>,
    mut button_query: Query<(&MenuButton, &Children, &mut BackgroundColor), With<Button>>,
    mut text_colors: Query<&mut TextColor>,
) {
    if modal_state.is_open {
        return;
    }

    for (button, children, mut background) in &mut button_query {
        let is_selected = button.index == selection.index;

        *background = BackgroundColor(Color::NONE);

        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                *text_color = if is_selected {
                    TextColor(Color::srgb(0.3, 0.6, 1.0))
                } else {
                    TextColor(Color::WHITE)
                };
            }
        }
    }
}

pub(crate) fn fit_background_to_window(
    windows: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut backgrounds: Query<(&Sprite, &mut Transform), With<StartScreenBackground>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    for (sprite, mut transform) in &mut backgrounds {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };

        let image_size = Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        );

        if image_size.x <= 0.0 || image_size.y <= 0.0 {
            continue;
        }

        let scale = (window_size.x / image_size.x).max(window_size.y / image_size.y);
        transform.scale = Vec3::splat(scale);
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

