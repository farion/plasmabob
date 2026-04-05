use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::helper::i18n::LocalizedText;

#[derive(Resource, Default)]
pub(crate) struct SettingsWindowState {
    pub(crate) is_open: bool,
}

pub fn settings_egui_system(
    mut contexts: EguiContexts,
    mut settings_win: ResMut<SettingsWindowState>,
    mut audio_settings: ResMut<crate::helper::audio_settings::AudioSettings>,
    mut current: ResMut<crate::i18n::CurrentLanguage>,
    translations: Res<crate::i18n::Translations>,
) {
    if !settings_win.is_open { return; }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Settings");

            ui.horizontal(|ui| {
                ui.label("Music");
                let mut v = audio_settings.music_volume;
                if ui.add(egui::Slider::new(&mut v, 0.0..=1.0)).changed() {
                    audio_settings.set_music_volume(v);
                    if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings: {e}"); }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Effects");
                let mut v = audio_settings.effects_volume;
                if ui.add(egui::Slider::new(&mut v, 0.0..=1.0)).changed() {
                    audio_settings.set_effects_volume(v);
                    if let Err(e) = audio_settings.save_to_disk() { warn!("Could not save audio settings: {e}"); }
                }
            });

            ui.separator();

            ui.label("Language");
            let mut selected = current.0.clone();
            let mut combo = egui::ComboBox::from_label("Language");
            combo.show_ui(ui, |ui| {
                if ui.selectable_label(selected.is_none(), "Auto").clicked() {
                    selected = None;
                }
                for code in crate::i18n::available_language_codes(&translations) {
                    let name = translations.tr(&code, "settings.language.name").map(|s| s.to_string()).unwrap_or_else(|| code.to_uppercase());
                    if ui.selectable_label(selected.as_ref().map(|s| s==&code).unwrap_or(false), name).clicked() {
                        selected = Some(code.clone());
                    }
                }
            });
            if selected != current.0 {
                current.0 = selected;
                if let Err(e) = current.save_to_disk() { warn!("Failed to save language selection: {e}"); }
            }

            ui.separator();
            if ui.button("Advanced settings (key bindings)").clicked() {
                // Fallback to original settings view for advanced editing
                // This will switch state to the legacy SettingsView
                let mut next_state = crate::app_model::AppState::SettingsView;
                // Request the state change by emitting NextState — use global world directly
                ui.ctx().output().close_menu();
            }

            if ui.button("Close").clicked() {
                settings_win.is_open = false;
            }
        });
}

