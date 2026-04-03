use bevy::prelude::*;

use crate::audio_settings::AudioSettings;
use crate::key_bindings::{KeyAction, KeyBindings};
use crate::AppState;
use crate::i18n::LocalizedText;

pub struct SettingsViewPlugin;

const VOLUME_STEP: f32 = 0.05;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VolumeSetting {
    Music,
    Effects,
    Quotes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsItem {
    Volume(VolumeSetting),
    Key(KeyAction),
}

const ALL_ITEMS: [SettingsItem; 8] = [
    SettingsItem::Volume(VolumeSetting::Music),
    SettingsItem::Volume(VolumeSetting::Effects),
    SettingsItem::Volume(VolumeSetting::Quotes),
    SettingsItem::Key(KeyAction::MoveLeft),
    SettingsItem::Key(KeyAction::MoveRight),
    SettingsItem::Key(KeyAction::Jump),
    SettingsItem::Key(KeyAction::Shoot),
    SettingsItem::Key(KeyAction::Fullscreen),
];

#[derive(Component)]
struct SettingsViewEntity;

#[derive(Component)]
struct VolumeRow { setting: VolumeSetting }

#[derive(Component)]
struct VolumeValueText { setting: VolumeSetting }

#[derive(Component)]
struct VolumeAdjustButton { setting: VolumeSetting, delta: f32 }

#[derive(Component)]
struct KeyBindingRow { action: KeyAction }

#[derive(Component)]
struct KeyBindingValueText { action: KeyAction }

#[derive(Component)]
struct KeyBindingButton { action: KeyAction }

#[derive(Component)]
struct ErrorMessage {
    timer: Timer,
}

#[derive(Resource)]
struct SettingsSelection { item: SettingsItem }

impl Default for SettingsSelection {
    fn default() -> Self {
        Self { item: SettingsItem::Volume(VolumeSetting::Music) }
    }
}

/// Wenn Some(action), wird auf den nächsten gültigen Tastendruck für diese Aktion gewartet.
#[derive(Resource, Default)]
struct BindingCapture(Option<KeyAction>);

impl Plugin for SettingsViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::SettingsView), setup_settings_view)
            .add_systems(
                Update,
                (
                    keyboard_controls,
                    mouse_controls,
                    refresh_settings_ui,
                    update_error_messages,
                    save_settings_on_change,
                    return_to_main_menu,
                )
                    .run_if(in_state(AppState::SettingsView)),
            )
            .add_systems(
                OnExit(AppState::SettingsView),
                (save_settings_on_exit, cleanup_settings_view),
            );
    }
}

fn setup_settings_view(
    mut commands: Commands,
    audio_settings: Res<AudioSettings>,
    key_bindings: Res<KeyBindings>,
    asset_server: Res<AssetServer>,
) {
    // Background same as main menu
    commands.spawn((
        Sprite::from_image(asset_server.load("start.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
        crate::StartScreenBackground,
        SettingsViewEntity,
    ));
    commands.init_resource::<SettingsSelection>();
    commands.init_resource::<BindingCapture>();

    // Root container should fill the whole viewport so the settings view scales to the window.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            SettingsViewEntity,
        ))
        .with_children(|parent| {
            // Error-Message Container (oben)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                SettingsViewEntity,
            )).with_children(|error_parent| {
                error_parent.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    SettingsViewEntity,
                ));
            });

            parent.spawn((
                Text::new(""),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::WHITE),
                LocalizedText { key: "settings.title".to_string() },
                SettingsViewEntity,
            ));

            spawn_section_header(parent, "settings.section.volume");
            spawn_volume_row(parent, VolumeSetting::Music, audio_settings.music_volume);
            spawn_volume_row(parent, VolumeSetting::Effects, audio_settings.effects_volume);
            spawn_volume_row(parent, VolumeSetting::Quotes, audio_settings.quotes_volume);

            spawn_section_header(parent, "settings.section.keybindings");
            for action in KeyAction::all() {
                spawn_key_binding_row(parent, action, key_bindings.get(action));
            }

            parent.spawn((
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LocalizedText { key: "settings.hint".to_string() },
                SettingsViewEntity,
            ));
        });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, key: &str) {
    parent.spawn((
        Text::new(""),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.55, 0.75, 1.0)),
        LocalizedText { key: key.to_string() },
        SettingsViewEntity,
    ));
}

fn spawn_volume_row(parent: &mut ChildSpawnerCommands, setting: VolumeSetting, value: f32) {
    let label_key = match setting {
        VolumeSetting::Music => "settings.volume.music",
        VolumeSetting::Effects => "settings.volume.effects",
        VolumeSetting::Quotes => "settings.volume.quotes",
    };
    parent
        .spawn((
            Node {
                width: Val::Px(540.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            VolumeRow { setting },
            SettingsViewEntity,
        ))
        .with_children(|row| {
            row.spawn((Text::new(""), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE), LocalizedText { key: label_key.to_string() }, SettingsViewEntity));
            row.spawn((
                Button,
                Node { width: Val::Px(40.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                VolumeAdjustButton { setting, delta: -VOLUME_STEP },
                SettingsViewEntity,
            )).with_children(|b| { b.spawn((Text::new("-"), TextFont { font_size: 26.0, ..default() }, TextColor(Color::WHITE), SettingsViewEntity)); });
            row.spawn((Text::new(format_percent(value)), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE), VolumeValueText { setting }, SettingsViewEntity));
            row.spawn((
                Button,
                Node { width: Val::Px(40.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                VolumeAdjustButton { setting, delta: VOLUME_STEP },
                SettingsViewEntity,
            )).with_children(|b| { b.spawn((Text::new("+"), TextFont { font_size: 26.0, ..default() }, TextColor(Color::WHITE), SettingsViewEntity)); });
        });
}

fn spawn_key_binding_row(parent: &mut ChildSpawnerCommands, action: KeyAction, current_key: KeyCode) {
    parent
        .spawn((
            Node {
                width: Val::Px(540.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            KeyBindingRow { action },
            SettingsViewEntity,
        ))
        .with_children(|row| {
            row.spawn((Text::new(""), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE), LocalizedText { key: action.label_key().to_string() }, SettingsViewEntity));
            row.spawn((
                Button,
                Node {
                    min_width: Val::Px(150.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                KeyBindingButton { action },
                SettingsViewEntity,
            )).with_children(|b| {
                b.spawn((
                    Text::new(KeyBindings::display_name(current_key)),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                    KeyBindingValueText { action },
                    SettingsViewEntity,
                ));
            });
        });
}

fn keyboard_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection: ResMut<SettingsSelection>,
    mut audio_settings: ResMut<AudioSettings>,
    mut key_bindings: ResMut<KeyBindings>,
    mut capture: ResMut<BindingCapture>,
) {
    // Im Capture-Modus: nächste gültige Taste wird als Belegung übernommen
    if let Some(action) = capture.0 {
        for &key in keys.get_just_pressed() {
            if key == KeyCode::Escape {
                capture.0 = None;
                return;
            }
            if KeyBindings::is_valid_binding_key(key) {
                if key_bindings.is_key_already_bound(key, action) {
                    // Fehlermeldung anzeigen
                    spawn_error_message(&mut commands, "");
                } else {
                    key_bindings.set(action, key);
                    capture.0 = None;
                }
                return;
            }
        }
        return;
    }

    // Navigation
    let nav = if keys.just_pressed(KeyCode::ArrowDown) { 1i32 }
               else if keys.just_pressed(KeyCode::ArrowUp) { -1i32 }
               else { 0 };
    if nav != 0 {
        let idx = ALL_ITEMS.iter().position(|&i| i == selection.item).unwrap_or(0);
        selection.item = ALL_ITEMS[(idx as i32 + nav).rem_euclid(ALL_ITEMS.len() as i32) as usize];
    }

    // Lautstärke anpassen
    if let SettingsItem::Volume(setting) = selection.item {
        if keys.just_pressed(KeyCode::ArrowLeft) { apply_volume_delta(&mut audio_settings, setting, -VOLUME_STEP); }
        if keys.just_pressed(KeyCode::ArrowRight) { apply_volume_delta(&mut audio_settings, setting, VOLUME_STEP); }
    }

    // Capture starten
    if let SettingsItem::Key(action) = selection.item {
        if keys.just_pressed(KeyCode::Enter) {
            capture.0 = Some(action);
        }
    }
}

fn mouse_controls(
    mut audio_settings: ResMut<AudioSettings>,
    mut selection: ResMut<SettingsSelection>,
    mut capture: ResMut<BindingCapture>,
    volume_interactions: Query<(&Interaction, &VolumeAdjustButton), (Changed<Interaction>, With<Button>)>,
    binding_interactions: Query<(&Interaction, &KeyBindingButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in &binding_interactions {
        match *interaction {
            Interaction::Pressed => {
                selection.item = SettingsItem::Key(button.action);
                capture.0 = Some(button.action);
            }
            Interaction::Hovered => { selection.item = SettingsItem::Key(button.action); }
            Interaction::None => {}
        }
    }

    if capture.0.is_some() { return; }

    for (interaction, button) in &volume_interactions {
        match *interaction {
            Interaction::Pressed => {
                selection.item = SettingsItem::Volume(button.setting);
                apply_volume_delta(&mut audio_settings, button.setting, button.delta);
            }
            Interaction::Hovered => { selection.item = SettingsItem::Volume(button.setting); }
            Interaction::None => {}
        }
    }
}

fn refresh_settings_ui(
    audio_settings: Res<AudioSettings>,
    key_bindings: Res<KeyBindings>,
    selection: Res<SettingsSelection>,
    capture: Res<BindingCapture>,
    mut volume_rows: Query<(&VolumeRow, &mut BackgroundColor)>,
    mut key_rows: Query<(&KeyBindingRow, &mut BackgroundColor), Without<VolumeRow>>,
    mut volume_texts: Query<(&VolumeValueText, &mut Text)>,
    mut binding_texts: Query<(&KeyBindingValueText, &mut Text), Without<VolumeValueText>>,
    mut binding_buttons: Query<(&KeyBindingButton, &mut BackgroundColor), (Without<VolumeRow>, Without<KeyBindingRow>)>,
) {
    if !audio_settings.is_changed() && !key_bindings.is_changed()
        && !selection.is_changed() && !capture.is_changed()
    {
        return;
    }

    for (row, mut bg) in &mut volume_rows {
        *bg = BackgroundColor(if selection.item == SettingsItem::Volume(row.setting) {
            Color::srgba(0.2, 0.35, 0.7, 0.7)
        } else {
            Color::srgba(0.1, 0.1, 0.1, 0.6)
        });
    }

    for (row, mut bg) in &mut key_rows {
        *bg = BackgroundColor(if selection.item == SettingsItem::Key(row.action) {
            Color::srgba(0.2, 0.35, 0.7, 0.7)
        } else {
            Color::srgba(0.1, 0.1, 0.1, 0.6)
        });
    }

    for (vt, mut text) in &mut volume_texts {
        let value = match vt.setting {
            VolumeSetting::Music => audio_settings.music_volume,
            VolumeSetting::Effects => audio_settings.effects_volume,
            VolumeSetting::Quotes => audio_settings.quotes_volume,
        };
        *text = Text::new(format_percent(value));
    }

    for (bt, mut text) in &mut binding_texts {
        *text = Text::new(if capture.0 == Some(bt.action) {
            "...".to_string()
        } else {
            KeyBindings::display_name(key_bindings.get(bt.action)).to_string()
        });
    }

    for (button, mut bg) in &mut binding_buttons {
        *bg = BackgroundColor(if capture.0 == Some(button.action) {
            Color::srgba(0.7, 0.4, 0.1, 0.95)
        } else if selection.item == SettingsItem::Key(button.action) {
            Color::srgba(0.2, 0.2, 0.5, 0.9)
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9)
        });
    }
}

fn save_settings_on_change(audio_settings: Res<AudioSettings>, key_bindings: Res<KeyBindings>) {
    if audio_settings.is_changed() {
        if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings: {e}"); }
    }
    if key_bindings.is_changed() {
        if let Err(e) = key_bindings.save_to_disk() { warn!("Could not save key bindings: {e}"); }
    }
}

fn save_settings_on_exit(audio_settings: Res<AudioSettings>, key_bindings: Res<KeyBindings>) {
    if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings on exit: {e}"); }
    if let Err(e) = key_bindings.save_to_disk() { warn!("Could not save key bindings on exit: {e}"); }
}

fn return_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<BindingCapture>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if capture.0.is_some() { return; }
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn apply_volume_delta(audio_settings: &mut AudioSettings, setting: VolumeSetting, delta: f32) {
    match setting {
        VolumeSetting::Music => { audio_settings.set_music_volume(audio_settings.music_volume + delta); }
        VolumeSetting::Effects => { audio_settings.set_effects_volume(audio_settings.effects_volume + delta); }
        VolumeSetting::Quotes => { audio_settings.set_quotes_volume(audio_settings.quotes_volume + delta); }
    }
}

fn format_percent(value: f32) -> String {
    format!("{}%", (value * 100.0).round() as i32)
}

fn spawn_error_message(commands: &mut Commands, _message: &str) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Auto,
                right: Val::Auto,
                width: Val::Auto,
                height: Val::Auto,
                padding: UiRect::axes(Val::Px(24.0), Val::Px(14.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.2, 0.2, 0.95)),
            Text::new(""),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
            LocalizedText { key: "settings.error.key_already_bound".to_string() },
            ErrorMessage {
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            },
            SettingsViewEntity,
        ));
}

fn update_error_messages(
    mut commands: Commands,
    time: Res<Time>,
    mut error_messages: Query<(Entity, &mut ErrorMessage)>,
) {
    for (entity, mut error) in &mut error_messages {
        error.timer.tick(time.delta());
        if error.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_settings_view(mut commands: Commands, entities: Query<Entity, (With<SettingsViewEntity>, Without<ChildOf>)>) {
    commands.remove_resource::<SettingsSelection>();
    commands.remove_resource::<BindingCapture>();
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
