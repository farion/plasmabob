// audio imports for menu music are handled in `views::main_view` now
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use avian2d::{math::Vector, prelude::{Gravity, PhysicsPlugins}};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use crate::helper::audio_settings::AudioSettings;
use world::WorldCatalog;
use crate::helper::key_bindings as key_bindings;
use crate::helper::i18n as i18n;
use crate::helper::fonts as fonts;

mod helper;
mod game;
mod views;
pub(crate) mod level;
pub(crate) mod world;

const SHOW_HITBOX_DEBUG_LINES: bool = false;

// keys into the i18n JSON files
pub(crate) const MENU_ITEMS: [(&str, MenuAction); 4] = [
    ("menu.start", MenuAction::Start),
    ("menu.settings", MenuAction::Settings),
    ("menu.about", MenuAction::About),
    ("menu.exit", MenuAction::Exit),
];

pub(crate) const EXIT_CONFIRM_ITEMS: [(&str, ExitConfirmAction); 2] = [
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
pub(crate) enum MenuAction {
    Start,
    Settings,
    About,
    Exit,
}

#[derive(Clone, Copy)]
pub(crate) enum ExitConfirmAction {
    Confirm,
    Cancel,
}

#[derive(Resource, Default)]
pub(crate) struct MenuSelection {
    pub(crate) index: usize,
}

#[derive(Resource, Default)]
pub(crate) struct ExitConfirmModalState {
    pub(crate) is_open: bool,
    pub(crate) selection: usize,
    pub(crate) suppress_enter_until_release: bool,
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
pub(crate) struct MainMenuEntity;

#[derive(Component)]
pub(crate) struct MenuMusicEntity;

#[derive(Component)]
pub(crate) struct MenuButton {
    pub(crate) index: usize,
    pub(crate) action: MenuAction,
}

#[derive(Component)]
pub(crate) struct ExitConfirmModalRoot;

#[derive(Component)]
pub(crate) struct ExitConfirmButton {
    pub(crate) index: usize,
    pub(crate) action: ExitConfirmAction,
}

#[derive(Component)]
pub(crate) struct StartScreenBackground;

#[derive(Component)]
pub(crate) struct MainCamera;

fn main() {
    let level_selection = LevelSelection::from_cli_arg(std::env::args().nth(1));
    let cached_level_definition = level::CachedLevelDefinition::empty();
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
        .insert_resource(i18n::CurrentLanguage::load_from_disk())
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
        .add_plugins(views::ViewsPlugin)
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
