use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use avian2d::{math::Vector, prelude::{Gravity, PhysicsPlugins}};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use crate::audio_settings::AudioSettings;
use crate::game::world::WorldCatalog;

mod audio_settings;
mod fonts;
mod game;
mod key_bindings;
mod views;
mod i18n;

const SHOW_HITBOX_DEBUG_LINES: bool = false;

// keys into the i18n JSON files
const MENU_ITEMS: [(&str, MenuAction); 4] = [
    ("menu.start", MenuAction::Start),
    ("menu.settings", MenuAction::Settings),
    ("menu.about", MenuAction::About),
    ("menu.exit", MenuAction::Exit),
];

const EXIT_CONFIRM_ITEMS: [(&str, ExitConfirmAction); 2] = [
    ("modal.exit.yes", ExitConfirmAction::Confirm),
    ("modal.exit.no", ExitConfirmAction::Cancel),
];

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub(crate) enum AppState {
    #[default]
    MainMenu,
    StartView,
    StoryView,
    WorldMapView,
    GameView,
    LoseView,
    WinView,
    LoadView,
    SettingsView,
    AboutView,
}

#[derive(Clone, Copy)]
enum MenuAction {
    Start,
    Settings,
    About,
    Exit,
}

#[derive(Clone, Copy)]
enum ExitConfirmAction {
    Confirm,
    Cancel,
}

#[derive(Resource, Default)]
struct MenuSelection {
    index: usize,
}

#[derive(Resource, Default)]
struct ExitConfirmModalState {
    is_open: bool,
    selection: usize,
    suppress_enter_until_release: bool,
}

#[derive(Resource, Default)]
pub(crate) struct WorldListSelection {
    pub(crate) index: usize,
}

#[derive(Resource, Default)]
pub(crate) struct WorldMapSelection {
    pub(crate) index: usize,
}

#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct CampaignProgress {
    pub(crate) world_index: Option<usize>,
    pub(crate) planet_index: Option<usize>,
    pub(crate) level_index: usize,
    pub(crate) world_start_story_seen: bool,
}

#[derive(Resource, Debug, Default, Clone)]
pub(crate) struct LevelStats {
    pub(crate) enemies_killed: u32,
    pub(crate) total_time_seconds: f32,
    pub(crate) jumps: u32,
    pub(crate) shots: u32,
    pub(crate) hits: u32,
}

impl CampaignProgress {
    pub(crate) fn clear_planet_progress(&mut self) {
        self.planet_index = None;
        self.level_index = 0;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StoryScreenRequest {
    pub(crate) text_asset_path: String,
    pub(crate) background_asset_path: String,
    pub(crate) continue_to: AppState,
}

#[derive(Resource, Debug, Default, Clone)]
pub(crate) struct PendingStoryScreen {
    current: Option<StoryScreenRequest>,
}

impl PendingStoryScreen {
    pub(crate) fn set(&mut self, request: StoryScreenRequest) {
        self.current = Some(request);
    }

    pub(crate) fn take(&mut self) -> Option<StoryScreenRequest> {
        self.current.take()
    }
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct LevelSelection {
    asset_path: String,
}

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DebugRenderSettings {
    pub(crate) show_hitbox_lines: bool,
    pub(crate) show_overlay: bool,
}

impl LevelSelection {
    fn normalize_asset_path(raw: &str) -> String {
        let trimmed = raw.trim().trim_start_matches("assets/");

        if trimmed.ends_with(".json") {
            trimmed.to_string()
        } else {
            format!("{trimmed}.json")
        }
    }

    fn from_cli_arg(arg: Option<String>) -> Self {
        let raw = arg.unwrap_or_else(|| "level1.json".to_string());
        let trimmed = raw.trim().trim_start_matches("assets/");

        let with_folder = if trimmed.starts_with("levels/") {
            trimmed.to_string()
        } else {
            format!("levels/{trimmed}")
        };

        let asset_path = Self::normalize_asset_path(&with_folder);

        Self { asset_path }
    }

    pub(crate) fn set_asset_path(&mut self, raw: &str) {
        self.asset_path = Self::normalize_asset_path(raw);
    }

    pub(crate) fn asset_path(&self) -> &str {
        &self.asset_path
    }
}

#[derive(Component)]
struct MainMenuEntity;

#[derive(Component)]
struct MenuMusicEntity;

#[derive(Component)]
struct MenuButton {
    index: usize,
    action: MenuAction,
}

#[derive(Component)]
struct ExitConfirmModalRoot;

#[derive(Component)]
struct ExitConfirmButton {
    index: usize,
    action: ExitConfirmAction,
}

#[derive(Component)]
struct StartScreenBackground;

#[derive(Component)]
pub(crate) struct MainCamera;

fn main() {
    let level_selection = LevelSelection::from_cli_arg(std::env::args().nth(1));
    let cached_level_definition = game::level::CachedLevelDefinition::empty();
    let audio_settings = AudioSettings::load_from_disk();
    let key_bindings = key_bindings::KeyBindings::load_from_disk();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 1800.0))
        .insert_resource(FramepaceSettings {
            limiter: Limiter::from_framerate(60.0),
            ..default()
        })
        .insert_resource(DebugRenderSettings {
            show_hitbox_lines: SHOW_HITBOX_DEBUG_LINES,
            show_overlay: false,
        })
        .init_resource::<MenuSelection>()
        .init_resource::<ExitConfirmModalState>()
        .init_resource::<WorldListSelection>()
        .init_resource::<WorldMapSelection>()
        .init_resource::<CampaignProgress>()
        .init_resource::<LevelStats>()
        .init_resource::<PendingStoryScreen>()
        .init_resource::<i18n::Translations>()
        .init_resource::<i18n::CurrentLanguage>()
        .insert_resource(level_selection)
        .insert_resource(WorldCatalog::default())
        .insert_resource(cached_level_definition)
        .insert_resource(audio_settings)
        .insert_resource(key_bindings)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PlasmaBob".into(),
                // Start in borderless fullscreen by default
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                ..default()
            }),
            ..default()
        }))
        // Must come after DefaultPlugins so Assets<Font> already exists.
        // Replaces Bevy's default FiraMono with SpaceMono Regular globally.
        .add_plugins(fonts::FontsPlugin)
        .add_plugins(FramepacePlugin)
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .init_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, i18n::load_translations)
        .add_systems(Update, toggle_fullscreen)
        .add_systems(Update, i18n::update_localized_texts)
        // Always keep background sprites fitted to the current window size so views
        // that also spawn `StartScreenBackground` (About/Settings) are handled.
        .add_systems(Update, fit_background_to_window)
        .add_plugins(views::ViewsPlugin)
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnEnter(AppState::StartView), stop_menu_music)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(
            Update,
            (
                open_or_close_exit_modal_with_escape,
                menu_keyboard_navigation,
                menu_pointer_input,
                activate_selected_menu_item,
                update_menu_visuals,
                modal_keyboard_navigation,
                modal_pointer_input,
                activate_selected_modal_item,
                update_modal_visuals,
                sync_exit_modal,
            )
                .run_if(in_state(AppState::MainMenu)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<key_bindings::KeyBindings>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(key_bindings.fullscreen) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }
}

fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut selection: ResMut<MenuSelection>,
    mut modal_state: ResMut<ExitConfirmModalState>,
    mut world_catalog: ResMut<WorldCatalog>,
) {
    selection.index = 0;
    modal_state.is_open = false;
    modal_state.selection = 1;
    modal_state.suppress_enter_until_release = false;

    // Refresh world catalog early so the main menu can make decisions
    // (e.g. skip the StartView when only one world is present).
    world_catalog.refresh(&asset_server);

    commands.spawn((
        AudioPlayer::new(asset_server.load("music/start.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(audio_settings.music_volume),
            ..default()
        },
        MenuMusicEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("start.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
        MainMenuEntity,
        StartScreenBackground,
    ));

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
                            TextFont {
                                font_size: 46.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            i18n::LocalizedText { key: label.to_string() },
                            MainMenuEntity,
                        ));
                    });
            }
        });
}

fn cleanup_main_menu(mut commands: Commands, entities: Query<Entity, (With<MainMenuEntity>, Without<ChildOf>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn stop_menu_music(mut commands: Commands, music_entities: Query<Entity, With<MenuMusicEntity>>) {
    for entity in &music_entities {
        commands.entity(entity).despawn();
    }
}

fn menu_keyboard_navigation(
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

fn menu_pointer_input(
    interactions: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut selection: ResMut<MenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit: MessageWriter<AppExit>,
    world_catalog: Res<WorldCatalog>,
    mut progress: ResMut<CampaignProgress>,
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

fn activate_selected_menu_item(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit: MessageWriter<AppExit>,
    world_catalog: Res<WorldCatalog>,
    mut progress: ResMut<CampaignProgress>,
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
    world_catalog: &Res<WorldCatalog>,
    progress: &mut ResMut<CampaignProgress>,
    modal_state: &mut ResMut<ExitConfirmModalState>,
) {
    match action {
        MenuAction::Start => {
            let count = world_catalog.worlds().len();
            if count == 1 {
                // Skip world list and go directly to the single world's map
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
            // If Enter opened the modal, keep that keypress from selecting inside/outside the modal.
            modal_state.suppress_enter_until_release = true;
        }
    }
}

fn open_or_close_exit_modal_with_escape(
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

fn modal_keyboard_navigation(
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

fn modal_pointer_input(
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

fn activate_selected_modal_item(
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

fn sync_exit_modal(
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
                FocusPolicy::Block,
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
                            i18n::LocalizedText { key: "modal.exit.title".to_string() },
                            MainMenuEntity,
                        ));
                        panel.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.72, 0.72, 0.72)),
                            i18n::LocalizedText { key: "modal.exit.hint".to_string() },
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
                                                i18n::LocalizedText { key: label.to_string() },
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

fn update_modal_visuals(
    modal_state: Res<ExitConfirmModalState>,
    mut button_query: Query<(&ExitConfirmButton, &Children, &mut BackgroundColor), With<Button>>,
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

fn update_menu_visuals(
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

        // Keep menu buttons transparent; only text color indicates selection.
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


fn fit_background_to_window(
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

        // Use "cover" scaling so the background always fills the viewport.
        // This may crop the image on one axis but guarantees no black bars.
        let scale = (window_size.x / image_size.x).max(window_size.y / image_size.y);
        transform.scale = Vec3::splat(scale);

        // Ensure the sprite is centred on the camera so it aligns with the viewport.
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

