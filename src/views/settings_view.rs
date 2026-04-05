use bevy::prelude::*;

use crate::app_model::{AppState, StartScreenBackground};
use crate::helper::audio_settings::AudioSettings;
use crate::key_bindings::{KeyAction, KeyBindings};
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
    Language,
}

const ALL_ITEMS: [SettingsItem; 9] = [
    SettingsItem::Volume(VolumeSetting::Music),
    SettingsItem::Volume(VolumeSetting::Effects),
    SettingsItem::Volume(VolumeSetting::Quotes),
    SettingsItem::Key(KeyAction::MoveLeft),
    SettingsItem::Key(KeyAction::MoveRight),
    SettingsItem::Key(KeyAction::Jump),
    SettingsItem::Key(KeyAction::Shoot),
    SettingsItem::Key(KeyAction::Fullscreen),
    SettingsItem::Language,
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
struct LanguageButton {
    /// None == Auto, Some(code) == explicit language code like "en"
    code: Option<String>,
    index: usize,
}

#[derive(Component)]
struct LanguageToggleButton;

#[derive(Component)]
struct LanguageDropdownRoot;

#[derive(Component)]
struct LanguageToggleText;

#[derive(Component)]
struct LanguageRow;

#[derive(Resource, Default)]
struct LanguageDropdownState {
    is_open: bool,
    index: usize, // 0 == Auto, 1.. == languages
}

#[derive(Resource)]
struct LanguageOptions {
    /// ordered options: None == Auto, Some(code) == explicit language code
    options: Vec<Option<String>>,
}

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
        app.add_systems(OnEnter(AppState::SettingsView), setup_settings_view);
        app.add_systems(Update, keyboard_controls.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, language_keyboard_controls.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, mouse_controls.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, language_pointer_input.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, refresh_settings_ui.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, language_row_highlight.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, update_error_messages.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, save_settings_on_change.run_if(in_state(AppState::SettingsView)));
        app.add_systems(Update, return_to_main_menu.run_if(in_state(AppState::SettingsView)));
        app.add_systems(OnExit(AppState::SettingsView), (save_settings_on_exit, cleanup_settings_view));
    }
}

fn setup_settings_view(
    mut commands: Commands,
    audio_settings: Res<AudioSettings>,
    key_bindings: Res<KeyBindings>,
    asset_server: Res<AssetServer>,
    translations: Res<crate::i18n::Translations>,
    current: Res<crate::i18n::CurrentLanguage>,
) {
    // prepare language options resource before building UI to avoid borrowing `commands` inside UI spawn closure
    let mut codes = crate::i18n::available_language_codes(&translations);
    codes.sort();
    let mut opts: Vec<Option<String>> = Vec::with_capacity(codes.len() + 1);
    opts.push(None); // Auto
    for c in codes.iter() { opts.push(Some(c.clone())); }
    commands.init_resource::<LanguageDropdownState>();
    commands.insert_resource(LanguageOptions { options: opts.clone() });
    // Background same as main menu
    commands.spawn((
        Sprite::from_image(asset_server.load("start.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
        StartScreenBackground,
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

            spawn_section_header(parent, "settings.section.language");
            // Spawn the pulldown in the same order as other settings rows so it is reachable by keyboard navigation
            spawn_language_row(parent, &translations, &current, &opts);
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

fn spawn_language_row(parent: &mut ChildSpawnerCommands, translations: &crate::i18n::Translations, current: &crate::i18n::CurrentLanguage, options: &Vec<Option<String>>) {
    // Build list: first option is Auto, then all detected language codes
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
            LanguageRow,
            SettingsViewEntity,
        ))
        .with_children(|row| {
            row.spawn((Text::new(""), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE), LocalizedText { key: "settings.language.label".to_string() }, SettingsViewEntity));

            // Options container (toggle + dropdown panel)
            row.spawn((
                Node { width: Val::Auto, height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
                SettingsViewEntity,
            ))
            .with_children(|opts| {
                // Main toggle button showing current selection
                let display_text = if let Some(code) = &current.0 {
                    // Try to show the language's own name (e.g. "Deutsch" for de)
                    translations
                        .tr(code, "settings.language.name")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| code.to_uppercase())
                } else {
                    translations
                        .tr(&current.effective(&translations), "settings.language.auto")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Auto".to_string())
                };

                // Use the same button sizing/layout as key binding buttons so heights match
                // Use the exact same button node shape as key binding buttons so heights match
                opts.spawn((
                    Button,
                    Node {
                        min_width: Val::Px(150.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    LanguageToggleButton,
                    SettingsViewEntity,
                )).with_children(|b| {
                    // match key binding value text size for consistent visual height
                    b.spawn((Text::new(display_text), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE), LanguageToggleText, SettingsViewEntity));
                });

                // Dropdown panel (initially hidden)
                opts.spawn((
                    Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), padding: UiRect::all(Val::Px(6.0)), ..default() },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.08, 1.0)),
                    LanguageDropdownRoot,
                    Visibility::Hidden,
                    SettingsViewEntity,
                ))
                .with_children(|panel| {
                    // Options from provided ordered list (first is None == Auto)
                    for (i, opt) in options.iter().enumerate() {
                        let label = match opt {
                            None => translations.tr(&current.effective(&translations), "settings.language.auto").map(|s| s.to_string()).unwrap_or_else(|| "Auto".to_string()),
                            Some(code) => translations.tr(code, "settings.language.name").map(|s| s.to_string()).unwrap_or_else(|| code.to_uppercase()),
                        };
                        panel.spawn((
                            Button,
                            Node { min_width: Val::Px(180.0), height: Val::Px(28.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::NONE),
                            LanguageButton { code: opt.clone(), index: i },
                            SettingsViewEntity,
                        )).with_children(|b| {
                            b.spawn((Text::new(label), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE), SettingsViewEntity));
                        });
                    }
                });
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
    lang_state: Res<LanguageDropdownState>,
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
    // If the language dropdown is open, the arrow keys should control the dropdown
    // and must not change the settings selection. Early-return in that case.
    if selection.item == SettingsItem::Language && lang_state.is_open {
        return;
    }

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
    lang_interactions: Query<(&Interaction, &LanguageButton), (Changed<Interaction>, With<Button>)>,
    toggle_interactions: Query<(&Interaction, &LanguageToggleButton), (Changed<Interaction>, With<Button>)>,
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

    // Visual selection on hover for languages (also make the language row selectable)
    for (interaction, _button) in &lang_interactions {
        match *interaction {
            Interaction::Hovered => { selection.item = SettingsItem::Language; }
            Interaction::Pressed => { selection.item = SettingsItem::Language; }
            Interaction::None => {}
        }
    }

    // Also handle hovering/pressing the toggle button
    for (interaction, _toggle) in &toggle_interactions {
        match *interaction {
            Interaction::Hovered => { selection.item = SettingsItem::Language; }
            Interaction::Pressed => { selection.item = SettingsItem::Language; }
            Interaction::None => {}
        }
    }
}

fn language_keyboard_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut lang_state: ResMut<LanguageDropdownState>,
    options: Res<LanguageOptions>,
    mut dropdown_vis: Query<&mut Visibility, With<LanguageDropdownRoot>>,
    mut current: ResMut<crate::i18n::CurrentLanguage>,
    selection: Res<SettingsSelection>,
) {
    // Only act when the language row is selected
    if selection.item != SettingsItem::Language { return; }
    // Open the dropdown with Enter/Space when closed. When open, Enter confirms the
    // currently focused option (NumpadEnter also works). Space should only open/close,
    // not confirm.
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::NumpadEnter) {
        if !lang_state.is_open {
            // open -> set initial focus to the currently selected language
            lang_state.is_open = true;
            for mut vis in &mut dropdown_vis {
                *vis = Visibility::Visible;
            }
            // Try to focus the currently saved language in the options list
            lang_state.index = options.options.iter().position(|o| o == &current.0).unwrap_or(0);
            return;
        }

        // If the dropdown is already open, Enter/NumpadEnter confirms selection.
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            if let Some(opt) = options.options.get(lang_state.index) {
                current.0 = opt.clone();
                if let Err(e) = current.save_to_disk() { warn!("Failed to save language selection: {e}"); }
            }
            // close
            lang_state.is_open = false;
            for mut vis in &mut dropdown_vis { *vis = Visibility::Hidden; }
            return;
        }
        // If it was Space while open, ignore (do not confirm) so user can use arrows first.
    }

    if !lang_state.is_open { return; }

    // Navigate options while open
    if keys.just_pressed(KeyCode::ArrowDown) {
        lang_state.index = (lang_state.index + 1).min(options.options.len().saturating_sub(1));
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        if lang_state.index == 0 { lang_state.index = 0; } else { lang_state.index -= 1; }
    }

    // Escape closes
    if keys.just_pressed(KeyCode::Escape) {
        lang_state.is_open = false;
        for mut vis in &mut dropdown_vis { *vis = Visibility::Hidden; }
    }
}

fn refresh_settings_ui(
    audio_settings: Res<AudioSettings>,
    key_bindings: Res<KeyBindings>,
    selection: Res<SettingsSelection>,
    capture: Res<BindingCapture>,
    translations: Res<crate::i18n::Translations>,
    mut volume_rows: Query<(&VolumeRow, &mut BackgroundColor), (Without<KeyBindingRow>, Without<LanguageButton>)>,
    mut key_rows: Query<(&KeyBindingRow, &mut BackgroundColor), (Without<VolumeRow>, Without<LanguageButton>)>,
    mut volume_texts: Query<(&VolumeValueText, &mut Text), (Without<KeyBindingValueText>, Without<LanguageToggleText>)>,
    mut binding_texts: Query<(&KeyBindingValueText, &mut Text), (Without<VolumeValueText>, Without<LanguageToggleText>)>,
    mut binding_buttons: Query<(&KeyBindingButton, &mut BackgroundColor), (Without<VolumeRow>, Without<KeyBindingRow>, Without<LanguageButton>)>,
    current: Res<crate::i18n::CurrentLanguage>,
    lang_state: Res<LanguageDropdownState>,
    mut lang_buttons: Query<(&LanguageButton, &Children, &mut BackgroundColor), (With<Button>, Without<KeyBindingButton>, Without<VolumeRow>)>,
    mut toggle_texts: Query<&mut Text, (With<LanguageToggleText>, Without<VolumeValueText>, Without<KeyBindingValueText>)>,
    mut dropdown_vis: Query<&mut Visibility, With<LanguageDropdownRoot>>,
    mut text_colors: Query<&mut TextColor>,
) {
    if !audio_settings.is_changed() && !key_bindings.is_changed()
        && !selection.is_changed() && !capture.is_changed() && !current.is_changed() && !lang_state.is_changed()
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



    // Update language button visuals based on current selection
    for (button, children, mut bg) in &mut lang_buttons {
        // Determine visual state: either the persistent current selection, or the temporary keyboard focus
        // Only highlight the option that currently has keyboard focus. Do NOT
        // highlight the persistently saved language value — the toggle shows it
        // in text but the dropdown list highlights only the focused item.
        let is_focused = lang_state.is_open && button.index == lang_state.index;
        *bg = BackgroundColor(if is_focused { Color::srgba(0.2, 0.35, 0.7, 0.7) } else { Color::srgba(0.2, 0.2, 0.2, 0.9) });
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(child) {
                *tc = if is_focused { TextColor(Color::srgb(0.3, 0.6, 1.0)) } else { TextColor(Color::WHITE) };
            }
        }
    }

    // Update toggle text to reflect current selection using localized names when available
    for mut text in &mut toggle_texts {
        let new = if let Some(code) = &current.0 {
            translations
                .tr(code, "settings.language.name")
                .map(|s| s.clone())
                .unwrap_or_else(|| code.to_uppercase())
        } else {
            translations
                .tr(&current.effective(&translations), "settings.language.auto")
                .map(|s| s.clone())
                .unwrap_or_else(|| "Auto".to_string())
        };
        *text = Text::new(new);
    }

    // Keep dropdown panels hidden/visible as needed — no-op here, toggled by input system.
    for mut v in &mut dropdown_vis { let _ = &mut v; }
}

// Highlight the language row when it is selected so keyboard navigation shows the same
// visual cue as for other rows.
fn language_row_highlight(
    selection: Res<SettingsSelection>,
    mut language_rows: Query<&mut BackgroundColor, With<LanguageRow>>,
) {
    if !selection.is_changed() { return; }
    for mut bg in &mut language_rows {
        *bg = BackgroundColor(if selection.item == SettingsItem::Language {
            Color::srgba(0.2, 0.35, 0.7, 0.7)
        } else {
            Color::srgba(0.1, 0.1, 0.1, 0.6)
        });
    }
}

fn save_settings_on_change(
    audio_settings: Res<AudioSettings>,
    key_bindings: Res<KeyBindings>,
    current: Res<crate::i18n::CurrentLanguage>,
) {
    if audio_settings.is_changed() {
        if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings: {e}"); }
    }
    if key_bindings.is_changed() {
        if let Err(e) = key_bindings.save_to_disk() { warn!("Could not save key bindings: {e}"); }
    }
    if current.is_changed() {
        if let Err(e) = current.save_to_disk() { warn!("Could not save language selection: {e}"); }
    }
}

fn save_settings_on_exit(audio_settings: Res<AudioSettings>, key_bindings: Res<KeyBindings>, current: Res<crate::i18n::CurrentLanguage>) {
    if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings on exit: {e}"); }
    if let Err(e) = key_bindings.save_to_disk() { warn!("Could not save key bindings on exit: {e}"); }
    if let Err(e) = current.save_to_disk() { warn!("Could not save language selection on exit: {e}"); }
}

fn return_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<BindingCapture>,
    mut lang_state: ResMut<LanguageDropdownState>,
    mut dropdown_vis: Query<&mut Visibility, With<LanguageDropdownRoot>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // If we're currently capturing a key for rebinding, ignore ESC here.
    if capture.0.is_some() { return; }

    if keys.just_pressed(KeyCode::Escape) {
        // If the language dropdown is open, close it instead of leaving the view.
        if lang_state.is_open {
            lang_state.is_open = false;
            for mut vis in &mut dropdown_vis { *vis = Visibility::Hidden; }
            return;
        }
        // Otherwise, return to main menu as before.
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

fn language_pointer_input(
    interactions: Query<(&Interaction, &LanguageButton, &ChildOf), (Changed<Interaction>, With<Button>)>,
    toggle_interactions: Query<(&Interaction, &LanguageToggleButton, &ChildOf), (Changed<Interaction>, With<Button>)>,
    mut dropdown_vis: Query<&mut Visibility, With<LanguageDropdownRoot>>,
    mut current: ResMut<crate::i18n::CurrentLanguage>,
    mut lang_state: ResMut<LanguageDropdownState>,
    options: Res<LanguageOptions>,
) {
    // Handle language option presses
    for (interaction, button, _parent) in &interactions {
        match *interaction {
            Interaction::Pressed => {
                current.0 = button.code.clone();
                if let Err(e) = current.save_to_disk() {
                    warn!("Failed to save language selection: {e}");
                }
                // Hide all dropdown panels
                for mut vis in &mut dropdown_vis {
                    *vis = Visibility::Hidden;
                }
            }
            Interaction::Hovered | Interaction::None => {}
        }
    }

    // Handle toggle presses (show/hide dropdown)
    for (interaction, _toggle, _parent) in &toggle_interactions {
        if *interaction != Interaction::Pressed { continue; }
        // Toggle all dropdown panels (there is typically only one)
        for mut vis in &mut dropdown_vis {
            match *vis {
                Visibility::Hidden => {
                    *vis = Visibility::Visible;
                    lang_state.is_open = true;
                    // focus the currently selected language when opening via pointer
                    lang_state.index = options.options.iter().position(|o| o == &current.0).unwrap_or(0);
                }
                Visibility::Visible => {
                    *vis = Visibility::Hidden;
                    lang_state.is_open = false;
                }
                _ => {*vis = Visibility::Hidden; lang_state.is_open = false;}
            }
        }
    }
}

