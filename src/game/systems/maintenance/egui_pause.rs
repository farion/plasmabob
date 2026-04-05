use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::time::Virtual;

use crate::app_model::AppState;
use crate::CampaignProgress;
use crate::game::systems::systems_api::GameViewEntity;

#[derive(Component)]
pub(crate) struct PauseEguiRoot;

pub(crate) fn update_pause_menu(
    mut egui_ctx: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    mut pause_menu_state: ResMut<crate::game::game_view::PauseMenuState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut progress: ResMut<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = egui_ctx.ctx_mut();

    // Toggle with Escape as before
    if keys.just_pressed(KeyCode::Escape) {
        pause_menu_state.is_open = !pause_menu_state.is_open;
        pause_menu_state.selection = 0;
    }

    if !pause_menu_state.is_open {
        virtual_time.unpause();
        return;
    }

    virtual_time.pause();

    // Draw pause window
    egui::Window::new("Paused")
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Paused");
            if ui.button("Resume").clicked() {
                pause_menu_state.is_open = false;
                virtual_time.unpause();
            }
            if ui.button("Settings").clicked() {
                // open settings view
                // Keep pause open; the SettingsView will be opened by state change in original code.
            }
            if ui.button("Restart Level").clicked() {
                pause_menu_state.is_open = false;
                virtual_time.unpause();
                next_state.set(AppState::LoadView);
            }
            if ui.button("Back to World Map").clicked() {
                pause_menu_state.is_open = false;
                progress.clear_planet_progress();
                virtual_time.unpause();
                next_state.set(AppState::WorldMapView);
            }
            if ui.button("Back to Main Menu").clicked() {
                pause_menu_state.is_open = false;
                progress.world_index = None;
                progress.clear_planet_progress();
                progress.world_start_story_seen = false;
                virtual_time.unpause();
                next_state.set(AppState::MainMenu);
            }
        });

    // keyboard navigation simplified: Enter resumes
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        pause_menu_state.is_open = false;
        virtual_time.unpause();
    }
}

