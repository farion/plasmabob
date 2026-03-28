use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use avian2d::{math::Vector, prelude::{Gravity, PhysicsPlugins}};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use crate::audio_settings::AudioSettings;

mod audio_settings;
mod game;
mod views;

const SHOW_HITBOX_DEBUG_LINES: bool = false;

const MENU_ITEMS: [(&str, MenuAction); 4] = [
    ("Start", MenuAction::Start),
    ("Settings", MenuAction::Settings),
    ("About", MenuAction::About),
    ("Exit", MenuAction::Exit),
];

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub(crate) enum AppState {
    #[default]
    MainMenu,
    StartView,
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

#[derive(Resource, Default)]
struct MenuSelection {
    index: usize,
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
    fn from_cli_arg(arg: Option<String>) -> Self {
        let raw = arg.unwrap_or_else(|| "level1.json".to_string());
        let trimmed = raw.trim().trim_start_matches("assets/");

        let with_folder = if trimmed.starts_with("levels/") {
            trimmed.to_string()
        } else {
            format!("levels/{trimmed}")
        };

        let asset_path = if with_folder.ends_with(".json") {
            with_folder
        } else {
            format!("{with_folder}.json")
        };

        Self { asset_path }
    }

    pub(crate) fn asset_path(&self) -> &str {
        &self.asset_path
    }
}

#[derive(Component)]
struct MainMenuEntity;

#[derive(Component)]
struct MenuButton {
    index: usize,
    action: MenuAction,
}

#[derive(Component)]
struct StartScreenBackground;

#[derive(Component)]
pub(crate) struct MainCamera;

fn main() {
    let level_selection = LevelSelection::from_cli_arg(std::env::args().nth(1));
    let audio_settings = AudioSettings::load_from_disk();

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
        .insert_resource(level_selection)
        .insert_resource(audio_settings)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PlasmaBob".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FramepacePlugin)
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .init_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_plugins(views::ViewsPlugin)
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(
            Update,
            (
                fit_background_to_window,
                menu_keyboard_navigation,
                menu_pointer_input,
                activate_selected_menu_item,
                update_menu_visuals,
            )
                .run_if(in_state(AppState::MainMenu)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut selection: ResMut<MenuSelection>,
) {
    selection.index = 0;


    commands.spawn((
        Sprite::from_image(asset_server.load("start.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
        MainMenuEntity,
        StartScreenBackground,
    ));

    commands
        .spawn((
            Node {
                width: Val::Px(320.0),
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
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
                            Text::new(label),
                            TextFont {
                                font_size: 46.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            MainMenuEntity,
                        ));
                    });
            }
        });
}

fn cleanup_main_menu(mut commands: Commands, entities: Query<Entity, With<MainMenuEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn menu_keyboard_navigation(keys: Res<ButtonInput<KeyCode>>, mut selection: ResMut<MenuSelection>) {
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
    mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, button) in &interactions {
        match *interaction {
            Interaction::Hovered => {
                selection.index = button.index;
            }
            Interaction::Pressed => {
                selection.index = button.index;
                activate_action(button.action, &mut next_state, &mut app_exit);
            }
            Interaction::None => {}
        }
    }
}

fn activate_selected_menu_item(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    if !keys.just_pressed(KeyCode::Enter) && !keys.just_pressed(KeyCode::NumpadEnter) {
        return;
    }

    let (_, action) = MENU_ITEMS[selection.index];
    activate_action(action, &mut next_state, &mut app_exit);
}

fn activate_action(
    action: MenuAction,
    next_state: &mut ResMut<NextState<AppState>>,
    app_exit: &mut EventWriter<AppExit>,
) {
    match action {
        MenuAction::Start => next_state.set(AppState::StartView),
        MenuAction::Settings => next_state.set(AppState::SettingsView),
        MenuAction::About => next_state.set(AppState::AboutView),
        MenuAction::Exit => {
            app_exit.send(AppExit::Success);
        }
    }
}

fn update_menu_visuals(
    selection: Res<MenuSelection>,
    mut button_query: Query<(&MenuButton, &Children, &mut BackgroundColor), With<Button>>,
    mut text_colors: Query<&mut TextColor>,
) {
    for (button, children, mut background) in &mut button_query {
        let is_selected = button.index == selection.index;

        // Keep menu buttons transparent; only text color indicates selection.
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

fn fit_background_to_window(
    windows: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut backgrounds: Query<(&Sprite, &mut Transform), With<StartScreenBackground>>,
) {
    let window = windows.single();

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

        // Use "contain" scaling to avoid distortion and show black bars when needed.
        let scale = (window_size.x / image_size.x).min(window_size.y / image_size.y);
        transform.scale = Vec3::splat(scale);
    }
}

