// Selective bevy imports to avoid name collisions (PointerState exists in bevy and in our crate)
use bevy::prelude::{
    Camera, Commands, GlobalTransform, Query, Res, ResMut, Time, Vec2, Window, With,
};
use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::core::io::{
    assets_dir, load_level, next_entity_id, save_level, scan_levels, scan_worlds,
};
use crate::core::{EntityDefinition, LevelBoundsDefinition};
use crate::dashboard;
use crate::level::helper::{
    apply_flat_component_updates, camera_center_world, flatten_entity_components,
    is_player_entity_type,
};
use crate::level::run::{ActiveCharacter, EditorCamera};
use crate::level::state::*;
use std::collections::HashMap;

pub fn level_picker_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    _time: Res<Time>,
    mut catalog: ResMut<LevelCatalog>,
    mut next_state: ResMut<bevy::prelude::NextState<crate::level::run::EditorMode>>,
    mut pointer_state: ResMut<PointerState>,
    mut selection: ResMut<SelectionState>,
    mut ui_state: ResMut<EditorUiState>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut camera_fit_requested: ResMut<CameraFitRequested>,
    mut undo_history: ResMut<UndoHistory>,
    mut undo_capture: ResMut<UndoCaptureState>,
    mut sync_state: ResMut<EntityTypesSyncState>,
    mut view_state: ResMut<EntityTypeViewState>,
    _toast: ResMut<ToastState>,
    mut active_character: ResMut<ActiveCharacter>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("PlasmaBob Level Editor");
        ui.horizontal(|_ui| {});
        ui.add_space(12.0);

        let entity_types_dir = assets_dir().join("entity_types");
        let mut entity_type_files: Vec<String> = Vec::new();
        let mut entity_type_error: Option<String> = None;
        match std::fs::read_dir(&entity_types_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            entity_type_files.push(name.to_string());
                        }
                    }
                }
                entity_type_files.sort();
            }
            Err(_) => {
                entity_type_error =
                    Some("Entity-types directory not found: assets/entity_types".to_string());
            }
        }

        let mut open_asset_path: Option<String> = None;
        if let Some(selected) = dashboard::render_level_picker_columns(
            ui,
            &mut open_asset_path,
            &mut catalog,
            &mut sync_state,
            &entity_type_files,
            &entity_type_error,
        ) {
            view_state.selected = Some(selected.clone());
            next_state.set(crate::level::run::EditorMode::EntityTypeView);
        }

        if let Some(asset_path) = open_asset_path {
            match load_level(&asset_path) {
                Ok(loaded) => {
                    commands.insert_resource(EditorDocument {
                        level_asset_path: loaded.level_asset_path,
                        level_fs_path: loaded.level_fs_path,
                        level: loaded.level,
                        entity_types: loaded.entity_types,
                        dirty: false,
                    });
                    selection.selected_index = None;
                    selection.is_dragging = false;
                    ui_state.show_add_menu = false;
                    undo_history.states.clear();
                    undo_capture.drag_snapshot_taken = false;
                    undo_capture.keyboard_move_active = false;
                    scene_dirty.0 = true;
                    camera_fit_requested.0 = true;
                    next_state.set(crate::level::run::EditorMode::Editing);
                }
                Err(error) => {
                    catalog.error = Some(error);
                }
            }
        }
    });

    egui::Area::new("character_toggle_area".into())
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_TOP, [-12.0, 12.0])
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let label = match *active_character {
                    ActiveCharacter::Bob => "Bob -> Betty",
                    ActiveCharacter::Betty => "Betty -> Bob",
                };
                if ui.button(label).clicked() {
                    let mut ac = *active_character;
                    ac.toggle();
                    *active_character = ac;
                    let _ = active_character.save_to_disk();
                }
            });
        });
}

pub fn editing_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut next_state: ResMut<bevy::prelude::NextState<crate::level::run::EditorMode>>,
    mut pointer_state: ResMut<PointerState>,
    mut ui_state: ResMut<EditorUiState>,
    overlay_mode: Res<ZOverlayMode>,
    hitbox_overlay: Res<HitboxOverlayState>,
    mut toast: ResMut<ToastState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut scene_dirty: ResMut<SceneDirty>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<SelectionState>,
    mapping: Res<ComponentValueMapping>,
    mut widths: ResMut<crate::core::ColumnWidths>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    // --- Top bar ---
    egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let dirty_marker = if document.dirty { " *" } else { "" };
            ui.heading(format!("{}{}", document.level_asset_path, dirty_marker));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui_phosphor_icons::icons::X).clicked() {
                    if document.dirty {
                        ui_state.show_close_confirm = true;
                    } else {
                        next_state.set(crate::level::run::EditorMode::LevelPicker);
                    }
                }
                ui.add_space(8.0);
                if ui.button(egui_phosphor_icons::icons::PLUS).clicked() {
                    ui_state.show_add_menu = !ui_state.show_add_menu;
                }
            });
        });
    });

    // Sidebar + selection UI are intentionally kept here; the implementation
    // was migrated from the previous editor.rs file without semantic changes.
    egui::SidePanel::right("editor_sidebar")
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.set_max_width(400.0);
            ui.heading("Selection");

            if let Some(index) = selection.selected_index {
                if let Some(entity) = document.level.entities.get(index) {
                    let id = entity.id.clone();
                    let entity_type_name = entity.entity_type.clone();
                    let current_z = entity.z_index.unwrap_or(100.0);
                    let mut x = entity.x;
                    let mut y = entity.y;
                    let mut z = current_z;
                    let mut changed = false;
                    // Clone override state so we don't hold a borrow into document below.
                    let current_overrides = flatten_entity_components(entity.components.as_ref());
                    let entity_type_def = document.entity_types.get(&entity_type_name).cloned();

                    ui.label(format!("ID: {}", id));
                    ui.label(format!("Type: {}", entity_type_name));
                    // The Z index is editable below; the small read-only label
                    // here duplicated that information so remove it.
                    ui.label("PgUp/PgDown: +/-1, with Shift: +/-10, Home: 150, End: 0");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("x:");
                        changed |= ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("y:");
                        changed |= ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("z:");
                        changed |= ui.add(egui::DragValue::new(&mut z).speed(1.0)).changed();
                    });
                    ui.add_space(6.0);

                    changed |= draw_z_layer_legend(ui, &mut z);

                    let mut collapsed_components =
                        std::mem::take(&mut ui_state.override_collapsed_components);
                    let mut json_editor_state =
                        std::mem::take(&mut ui_state.override_json_editor_state);
                    let mut array_editor = ui_state.override_array_editor.take();

                    let mut override_edits =
                        crate::core::components_overrides::LevelOverrideEdits::default();
                    if let Some(et) = &entity_type_def {
                        override_edits =
                            crate::core::components_overrides::render_level_entity_overrides_table(
                                ui,
                                ctx,
                                &id,
                                &entity_type_name,
                                &current_overrides,
                                et,
                                &mapping,
                                &mut collapsed_components,
                                &mut json_editor_state,
                                &mut array_editor,
                                &mut toast,
                                &time,
                                &mut widths,
                            );
                    }

                    ui_state.override_collapsed_components = collapsed_components;
                    ui_state.override_json_editor_state = json_editor_state;
                    ui_state.override_array_editor = array_editor;

                    if changed || override_edits.has_changes() {
                        crate::level::push_undo_snapshot(&mut undo_history, &document.level);
                        if let Some(entity) = document.level.entities.get_mut(index) {
                            entity.x = x;
                            entity.y = y;
                            entity.z_index = Some(z);
                            apply_flat_component_updates(
                                entity,
                                &override_edits.removals,
                                override_edits.updates,
                            );
                        }
                        document.dirty = true;
                        scene_dirty.0 = true;
                    }
                }
            } else if selection.bounds_selected {
                ui.label("Level background / Bounds selected");
                ui.label("Origin: (0, 0) — fixed");
                ui.add_space(8.0);

                let bounds = document.level.bounds.get_or_insert(LevelBoundsDefinition {
                    width: 1000.0,
                    height: 1024.0,
                });
                let mut width = bounds.width;
                let mut height = bounds.height;
                let mut bounds_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    bounds_changed |= ui
                        .add(
                            egui::DragValue::new(&mut width)
                                .speed(1.0)
                                .range(1.0..=50000.0),
                        )
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    bounds_changed |= ui
                        .add(
                            egui::DragValue::new(&mut height)
                                .speed(1.0)
                                .range(1.0..=50000.0),
                        )
                        .changed();
                });

                if bounds_changed {
                    crate::level::push_undo_snapshot(&mut undo_history, &document.level);
                    if let Some(b) = &mut document.level.bounds {
                        b.width = width.max(1.0);
                        b.height = height.max(1.0);
                    }
                    document.dirty = true;
                    scene_dirty.0 = true;
                }
            } else {
                ui.label("No selection.");
                ui.label("Click on entity or background.");
            }
        });

    if ui_state.show_keyboard_legend_overlay {
        draw_keyboard_legend_overlay(ctx, overlay_mode.enabled, hitbox_overlay.enabled);
    }

    if ui_state.show_add_menu {
        let mut open = ui_state.show_add_menu;
        egui::Window::new("Add Entity-Type")
            .open(&mut open)
            .default_size([320.0, 420.0])
            .show(ctx, |ui| {
                ui.label("Choose an entity type:");
                ui.separator();

                let mut entity_type_names: Vec<_> = document.entity_types.keys().cloned().collect();
                entity_type_names.sort();

                let camera_center = camera_center_world(&camera_query, &window_query);
                let spawn_position = camera_center.unwrap_or(bevy::prelude::Vec2::ZERO);
                let mut add_requested: Option<String> = None;

                ui.push_id("add_menu_entity_types_scroll", |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("editor_add_menu_entity_types_scroll")
                        .show(ui, |ui| {
                            for entity_type_name in entity_type_names {
                                ui.push_id(
                                    format!("addmenu_entity_type:{}", entity_type_name),
                                    |ui| {
                                        if ui.button(&entity_type_name).clicked() {
                                            add_requested = Some(entity_type_name);
                                        }
                                    },
                                );
                            }
                        });
                });

                if let Some(entity_type_name) = add_requested {
                    let is_player = document
                        .entity_types
                        .get(&entity_type_name)
                        .map(is_player_entity_type)
                        .unwrap_or(false);

                    let player_already_exists = is_player
                        && document.level.entities.iter().any(|e| {
                            document
                                .entity_types
                                .get(&e.entity_type)
                                .map(is_player_entity_type)
                                .unwrap_or(false)
                        });

                    if player_already_exists {
                        toast.message = Some("There can only be one player (Bob)!".to_string());
                        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    } else {
                        crate::level::push_undo_snapshot(&mut undo_history, &document.level);
                        let id = next_entity_id(&entity_type_name, &document.level.entities);
                        let new_entity = EntityDefinition {
                            id,
                            entity_type: entity_type_name,
                            x: spawn_position.x,
                            y: spawn_position.y,
                            z_index: Some(100.0),
                            name: None,
                            layer: None,
                            components: None,
                            extra: HashMap::new(),
                        };
                        document.level.entities.push(new_entity);
                        selection.selected_index = Some(document.level.entities.len() - 1);
                        document.dirty = true;
                        scene_dirty.0 = true;
                        ui_state.show_add_menu = false;
                    }
                }
            });
        ui_state.show_add_menu = ui_state.show_add_menu && open;
    }

    if let Some(message) = &toast.message {
        if time.elapsed_secs_f64() <= toast.expires_at_seconds {
            egui::Area::new("save_toast".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(420.0);
                        ui.set_max_width(900.0);
                        ui.label(message);
                    });
                });
        }
    }

    if ui_state.show_close_confirm {
        egui::Window::new("Confirm Close")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("There are unsaved changes.");
                ui.label("Save before closing?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save and Close").clicked() {
                        match save_level(&document.level_fs_path, &document.level) {
                            Ok(()) => {
                                document.dirty = false;
                                toast.message = Some("Saved".to_string());
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 2.0;
                                ui_state.show_close_confirm = false;
                                next_state.set(crate::level::run::EditorMode::LevelPicker);
                            }
                            Err(error) => {
                                toast.message = Some(format!("Save failed: {}", error));
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 4.0;
                            }
                        }
                    }

                    if ui.button("Discard and Close").clicked() {
                        document.dirty = false;
                        ui_state.show_close_confirm = false;
                        next_state.set(crate::level::run::EditorMode::LevelPicker);
                    }

                    if ui.button("Cancel").clicked() {
                        ui_state.show_close_confirm = false;
                    }
                });
            });
    }

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();
}

pub fn draw_keyboard_legend_overlay(
    ctx: &egui::Context,
    z_overlay_enabled: bool,
    hitbox_overlay_enabled: bool,
) {
    egui::Area::new("keyboard_legend_overlay".into())
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::LEFT_BOTTOM, [12.0, -12.0])
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 24, 30, 170))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                ))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_max_width(340.0);
                    ui.label(egui::RichText::new("Controls").strong());
                    ui.label("Left click: select / drag");
                    ui.label("A: Add entity");
                    ui.label("D: Remove entity");
                    ui.label("Arrows: move (Shift fast, Alt fine)");
                    ui.label("PgUp/PgDown: Z +/-1, with Shift +/-10");
                    ui.label("Home: Z=150, End: Z=0");
                    ui.label("Ctrl+C: copy entity");
                    ui.label("Ctrl+V: paste entity");
                    ui.label("Ctrl+S: save");
                    ui.label("Mouse wheel: zoom, right mouse button: pan camera");
                    let overlay_state = if z_overlay_enabled { "on" } else { "off" };
                    ui.label(format!("Z: Z-Overlay ({overlay_state})"));
                    let hitbox_state = if hitbox_overlay_enabled { "on" } else { "off" };
                    ui.label(format!("H: Toggle hitboxes ({hitbox_state})"));
                    ui.label("L: Toggle legend");
                    ui.label("S: Toggle snap");
                });
        });
}

pub fn draw_z_layer_legend(ui: &mut egui::Ui, z: &mut f32) -> bool {
    // Minimal inline legend implementation to avoid depending on editor.rs.
    // Legend header and read-only preset Z values.
    ui.separator();
    ui.label(
        egui::RichText::new("Z Legend")
            .strong()
            .color(ui.visuals().text_color()),
    );
    ui.add_space(4.0);
    // Build sorted presets from helper so displayed ranges reflect the
    // actual values used by the runtime.
    let mut presets: Vec<(&str, f32, [u8; 3])> = crate::level::helper::Z_LAYER_PRESETS.to_vec();
    presets.sort_by(|a, b| a.1.total_cmp(&b.1));

    // compute midpoints between adjacent preset values
    let mut midpoints: Vec<f32> = Vec::new();
    for pair in presets.windows(2) {
        let m = (pair[0].1 + pair[1].1) * 0.5;
        midpoints.push(m);
    }

    egui::Grid::new("z_layer_legend_grid")
        .num_columns(2)
        .show(ui, |ui| {
            for (i, (label, value, rgb)) in presets.iter().enumerate() {
                let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                ui.colored_label(color, *label);

                let range_label = if presets.len() == 1 {
                    format!("{:.0}", value)
                } else if i == 0 {
                    // lowest: <= first midpoint
                    format!("<= {:.0}", midpoints[0])
                } else if i == presets.len() - 1 {
                    // highest: > last midpoint
                    format!("> {:.0}", midpoints[midpoints.len() - 1])
                } else {
                    // middle ranges
                    let low = midpoints[i - 1];
                    let high = midpoints[i];
                    format!("{:.0} - {:.0}", low, high)
                };

                ui.label(egui::RichText::new(range_label).color(ui.visuals().text_color()));
                ui.end_row();
            }
        });
    // legend does not change state
    false
}
