// audio imports for menu music are handled in `views::main_view` now
use crate::app_model::{AppState, ExitConfirmModalState, MenuSelection};
use crate::helper::active_character::ActiveCharacter;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::fonts;
use crate::helper::i18n;
use crate::helper::key_bindings;
use crate::helper::music::MusicPlugin;
use crate::helper::sounds::SoundPlugin;
use avian2d::{
    math::Vector,
    prelude::{Gravity, PhysicsPlugins},
};
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use world::WorldCatalog;

mod app_model;
mod game;
mod helper;
mod views;
pub(crate) mod world;

const MAX_FRAME_RATE: f64 = 60.0;
const SHOW_HITBOX_DEBUG_LINES: bool = false;
const USE_PARALAX_SCROLLING: bool = true;

/// Virtual viewport width in world units. All game coordinates are designed for this width.
pub(crate) const VIRTUAL_WIDTH: f32 = 2048.0;
/// Virtual viewport height in world units.
pub(crate) const VIRTUAL_HEIGHT: f32 = 1536.0;
/// Horizontal screen anchor for the player: 0.3 = 30 % from the left edge.
pub(crate) const PLAYER_SCREEN_X: f32 = 0.3;

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
    pub(crate) collectibles_collected: u32,
    pub(crate) total_time_seconds: f32,
    pub(crate) jumps: u32,
    pub(crate) shots: u32,
    pub(crate) hits: u32,
    pub(crate) exit_bonus: u64,
    pub(crate) score: u64,
}

impl LevelStats {
    pub(crate) fn base_score(&self) -> u64 {
        (self.enemies_killed as u64)
            .saturating_mul(10)
            .saturating_add((self.collectibles_collected as u64).saturating_mul(20))
    }

    pub(crate) fn recompute_score(&mut self) {
        self.score = self.base_score().saturating_add(self.exit_bonus);
    }
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
    pub(crate) parallax_enabled: bool,
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
pub(crate) struct MainCamera;

fn main() {
    let level_selection = LevelSelection::from_cli_arg(std::env::args().nth(1));
    let audio_settings = AudioSettings::load_from_disk();
    let active_character = ActiveCharacter::load_from_disk();
    let key_bindings = key_bindings::KeyBindings::load_from_disk();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 1800.0))
        .insert_resource(FramepaceSettings {
            limiter: Limiter::from_framerate(MAX_FRAME_RATE),
            ..default()
        })
        .insert_resource(DebugRenderSettings {
            show_hitbox_lines: SHOW_HITBOX_DEBUG_LINES,
            parallax_enabled: USE_PARALAX_SCROLLING,
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
        .insert_resource(audio_settings)
        .insert_resource(active_character)
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
        .add_plugins(MusicPlugin)
        .add_plugins(SoundPlugin)
        .init_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, i18n::load_translations)
        .add_systems(Update, update_ui_scale)
        .add_systems(Update, toggle_fullscreen)
        .add_systems(Update, i18n::update_localized_texts)
        .add_plugins(views::ViewsPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Use AutoMin so that the 1024×768 virtual area is always fully visible.
    // On wider screens more world space shows on the sides; on taller screens
    // more shows top/bottom – but the virtual area is never clipped.
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIRTUAL_WIDTH,
                min_height: VIRTUAL_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
    ));
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

/// Keeps `UiScale` in sync with the physical window size so every `Val::Px`
/// value and every font size (all designed in virtual pixels for a
/// VIRTUAL_WIDTH × VIRTUAL_HEIGHT canvas) scale proportionally on any
/// screen resolution.
///
/// The scale factor is `min(window_w / VIRTUAL_WIDTH, window_h / VIRTUAL_HEIGHT)`
/// so that the entire virtual canvas always fits the window without cropping.
fn update_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = (window.width() / VIRTUAL_WIDTH).min(window.height() / VIRTUAL_HEIGHT);
    if scale > 0.0 && (ui_scale.0 - scale).abs() > f32::EPSILON {
        ui_scale.0 = scale;
    }
}
