use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource, Default)]
pub(crate) struct AboutWindowState {
    pub(crate) is_open: bool,
}

pub fn about_egui_system(
    mut contexts: EguiContexts,
    mut about_win: ResMut<AboutWindowState>,
    translations: Res<crate::i18n::Translations>,
    current: Res<crate::i18n::CurrentLanguage>,
) {
    if !about_win.is_open { return; }

    let ctx = contexts.ctx_mut();

    egui::Window::new("About")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("About");
            let title = translations.tr(&current.effective(&translations), "about.title").unwrap_or("About".into());
            let blurb = translations.tr(&current.effective(&translations), "about.blurb").unwrap_or("".into());
            ui.label(title);
            ui.separator();
            ui.label(blurb);
            ui.separator();
            if ui.button("Close").clicked() {
                about_win.is_open = false;
            }
        });
}

