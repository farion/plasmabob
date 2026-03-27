use bevy::prelude::*;

use crate::audio_settings::AudioSettings;
use crate::AppState;

pub struct SettingsViewPlugin;

const VOLUME_STEP: f32 = 0.05;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VolumeSetting {
    Music,
    Effects,
}

#[derive(Component)]
struct SettingsViewEntity;

#[derive(Component)]
struct VolumeRow {
    setting: VolumeSetting,
}

#[derive(Component)]
struct VolumeValueText {
    setting: VolumeSetting,
}

#[derive(Component)]
struct VolumeAdjustButton {
    setting: VolumeSetting,
    delta: f32,
}

#[derive(Resource)]
struct SettingsSelection {
    setting: VolumeSetting,
}

impl Default for SettingsSelection {
    fn default() -> Self {
        Self {
            setting: VolumeSetting::Music,
        }
    }
}

impl Plugin for SettingsViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::SettingsView), setup_settings_view)
            .add_systems(
                Update,
                (
                    keyboard_controls,
                    mouse_controls,
                    refresh_settings_ui,
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

fn setup_settings_view(mut commands: Commands, audio_settings: Res<AudioSettings>) {
    commands.init_resource::<SettingsSelection>();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            SettingsViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Einstellungen"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                SettingsViewEntity,
            ));

            spawn_volume_row(parent, VolumeSetting::Music, audio_settings.music_volume);
            spawn_volume_row(parent, VolumeSetting::Effects, audio_settings.effects_volume);

            parent.spawn((
                Text::new(
                    "Arrow Up/Down: Auswahl | Arrow Left/Right: Lautstaerke | Maus: +/- | Esc: Zurueck",
                ),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                SettingsViewEntity,
            ));
        });
}

fn spawn_volume_row(parent: &mut ChildBuilder, setting: VolumeSetting, value: f32) {
    let label = match setting {
        VolumeSetting::Music => "Musik",
        VolumeSetting::Effects => "Effekte (Quotes)",
    };

    parent
        .spawn((
            Node {
                width: Val::Px(540.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
            VolumeRow { setting },
            SettingsViewEntity,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                SettingsViewEntity,
            ));

            row.spawn((
                Button,
                Node {
                    width: Val::Px(44.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                VolumeAdjustButton {
                    setting,
                    delta: -VOLUME_STEP,
                },
                SettingsViewEntity,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("-"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SettingsViewEntity,
                ));
            });

            row.spawn((
                Text::new(format_percent(value)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                VolumeValueText { setting },
                SettingsViewEntity,
            ));

            row.spawn((
                Button,
                Node {
                    width: Val::Px(44.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                VolumeAdjustButton {
                    setting,
                    delta: VOLUME_STEP,
                },
                SettingsViewEntity,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("+"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SettingsViewEntity,
                ));
            });
        });
}

fn keyboard_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<SettingsSelection>,
    mut audio_settings: ResMut<AudioSettings>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) {
        selection.setting = match selection.setting {
            VolumeSetting::Music => VolumeSetting::Effects,
            VolumeSetting::Effects => VolumeSetting::Music,
        };
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        apply_volume_delta(&mut audio_settings, selection.setting, -VOLUME_STEP);
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        apply_volume_delta(&mut audio_settings, selection.setting, VOLUME_STEP);
    }
}

fn mouse_controls(
    mut audio_settings: ResMut<AudioSettings>,
    mut selection: ResMut<SettingsSelection>,
    interactions: Query<(&Interaction, &VolumeAdjustButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in &interactions {
        match *interaction {
            Interaction::Pressed => {
                selection.setting = button.setting;
                apply_volume_delta(&mut audio_settings, button.setting, button.delta);
            }
            Interaction::Hovered => {
                selection.setting = button.setting;
            }
            Interaction::None => {}
        }
    }
}

fn refresh_settings_ui(
    audio_settings: Res<AudioSettings>,
    selection: Res<SettingsSelection>,
    mut rows: Query<(&VolumeRow, &mut BackgroundColor)>,
    mut value_texts: Query<(&VolumeValueText, &mut Text)>,
) {
    if !audio_settings.is_changed() && !selection.is_changed() {
        return;
    }

    for (row, mut background) in &mut rows {
        let is_selected = row.setting == selection.setting;
        *background = if is_selected {
            BackgroundColor(Color::srgba(0.2, 0.35, 0.7, 0.7))
        } else {
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6))
        };
    }

    for (value_text, mut text) in &mut value_texts {
        let value = match value_text.setting {
            VolumeSetting::Music => audio_settings.music_volume,
            VolumeSetting::Effects => audio_settings.effects_volume,
        };
        *text = Text::new(format_percent(value));
    }
}

fn save_settings_on_change(audio_settings: Res<AudioSettings>) {
    if !audio_settings.is_changed() {
        return;
    }

    if let Err(error) = audio_settings.save_to_disk() {
        warn!("Could not save audio settings: {error}");
    }
}

fn save_settings_on_exit(audio_settings: Res<AudioSettings>) {
    if let Err(error) = audio_settings.save_to_disk() {
        warn!("Could not save audio settings on exit: {error}");
    }
}

fn return_to_main_menu(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn apply_volume_delta(audio_settings: &mut AudioSettings, setting: VolumeSetting, delta: f32) {
    match setting {
        VolumeSetting::Music => {
            let target = audio_settings.music_volume + delta;
            audio_settings.set_music_volume(target);
        }
        VolumeSetting::Effects => {
            let target = audio_settings.effects_volume + delta;
            audio_settings.set_effects_volume(target);
        }
    }
}

fn format_percent(value: f32) -> String {
    format!("{}%", (value * 100.0).round() as i32)
}

fn cleanup_settings_view(mut commands: Commands, entities: Query<Entity, With<SettingsViewEntity>>) {
    commands.remove_resource::<SettingsSelection>();

    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
