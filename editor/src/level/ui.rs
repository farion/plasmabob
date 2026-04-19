// Selective bevy imports to avoid name collisions (PointerState exists in bevy and in our crate)
use bevy::prelude::{Commands, With, Query, Res, ResMut, Time, Window, Camera, GlobalTransform, Vec2};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts};
use bevy_egui::egui;

use crate::io::{assets_dir, load_level, next_entity_id, save_level, scan_levels, scan_worlds};
use crate::dashboard;
use crate::entity_type;
use crate::model::{LevelBoundsDefinition, EntityDefinition};
use crate::level::state::*;
use crate::editor::{
    ActiveCharacter,
    SceneEntity,
    EditorCamera,
    entity_render_center,
    apply_flat_component_updates,
    z_overlay_color_for_value,
    entity_render_center as _entity_render_center_placeholder,
};
use std::collections::HashMap;

pub fn level_picker_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    _time: Res<Time>,
    mut catalog: ResMut<LevelCatalog>,
    mut next_state: ResMut<bevy::prelude::NextState<crate::editor::EditorMode>>,
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
    let Ok(ctx) = contexts.ctx_mut() else { return; };

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
                entity_type_error = Some("Entity-types directory not found: assets/entity_types".to_string());
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
            next_state.set(crate::editor::EditorMode::EntityTypeView);
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
                    next_state.set(crate::editor::EditorMode::Editing);
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
    mut next_state: ResMut<bevy::prelude::NextState<crate::editor::EditorMode>>,
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
    mut show_close_confirm: bevy::prelude::Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    // --- Top bar ---
    egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let dirty_marker = if document.dirty { " *" } else { "" };
            ui.heading(format!("{}{}", document.level_asset_path, dirty_marker));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui_phosphor_icons::icons::X).clicked() {
                    if document.dirty {
                        *show_close_confirm = true;
                    } else {
                        next_state.set(crate::editor::EditorMode::LevelPicker);
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
                    let current_overrides = crate::editor::flatten_entity_components(entity.components.as_ref());
                    let entity_type_def = document.entity_types.get(&entity_type_name).cloned();

                    ui.label(format!("ID: {}", id));
                    ui.label(format!("Type: {}", entity_type_name));
                    ui.label(format!("Z-Index: {}", current_z));
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

                    // --- Component Overrides ---
                    let mut override_updates: HashMap<String, serde_json::Value> = HashMap::new();
                    let mut override_removals: std::collections::HashSet<String> = Default::default();
                    let mut overrides_changed = false;

                    if let Some(et) = &entity_type_def {
                        let component_names = et.component_names();
                        let has_overrideable = component_names.iter()
                            .any(|comp| mapping.components.contains_key(comp.as_str()));

                        if has_overrideable {
                            ui.separator();
                            ui.label(egui::RichText::new("Overrides").strong());
                            ui.add_space(4.0);

                            for comp_name in &component_names {
                                let Some(attrs) = mapping.components.get(comp_name.as_str()) else { continue; };
                                let mut sorted_attrs: Vec<(&String, &ComponentAttributeDefinition)> = attrs.iter().collect();
                                sorted_attrs.sort_by_key(|(k, _)| k.as_str());

                                for (attr_name, attr_def) in sorted_attrs {
                                    let key = format!("{comp_name}.{attr_name}");

                                    let entity_type_default: serde_json::Value = et
                                        .component_attribute_value(comp_name.as_str(), attr_name.as_str())
                                        .unwrap_or_else(|| {
                                            match attr_def.attr_type.as_str() {
                                                "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                                                "enum" => serde_json::Value::String(
                                                    attr_def.options.get(0).cloned().unwrap_or_default(),
                                                ),
                                                _ => serde_json::Value::Null,
                                            }
                                        });

                                    let is_overridden = current_overrides.contains_key(&key);
                                    let mut enable_override = is_overridden;

                                    match attr_def.attr_type.as_str() {
                                        "number" => {
                                            let default_num = entity_type_default
                                                .as_f64()
                                                .unwrap_or(0.0)
                                                as f32;

                                            ui.horizontal(|ui| {
                                                let cb = ui.checkbox(&mut enable_override, format!("{key}:"));
                                                if cb.changed() {
                                                    if enable_override {
                                                        if let Some(n) = serde_json::Number::from_f64(default_num as f64) {
                                                            override_updates.insert(key.clone(), serde_json::Value::Number(n));
                                                        }
                                                    } else {
                                                        override_removals.insert(key.clone());
                                                    }
                                                    overrides_changed = true;
                                                }

                                                if enable_override {
                                                    let mut value = current_overrides
                                                        .get(&key)
                                                        .and_then(|v| v.as_f64())
                                                        .map(|v| v as f32)
                                                        .unwrap_or(default_num);
                                                    let before = value;
                                                    if ui.add(egui::DragValue::new(&mut value).speed(1.0)).changed()
                                                        && (value - before).abs() > f32::EPSILON
                                                    {
                                                        if let Some(n) = serde_json::Number::from_f64(value as f64) {
                                                            override_updates.insert(key, serde_json::Value::Number(n));
                                                            overrides_changed = true;
                                                        }
                                                    }
                                                } else {
                                                    ui.label(egui::RichText::new(format!("Type default: {default_num}")).weak().italics());
                                                }
                                            });
                                        }
                                        "enum" => {
                                            let default_str = entity_type_default
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string();

                                            ui.horizontal(|ui| {
                                                let cb = ui.checkbox(&mut enable_override, format!("{key}:"));
                                                if cb.changed() {
                                                    if enable_override {
                                                        override_updates.insert(key.clone(), serde_json::Value::String(default_str.clone()));
                                                    } else {
                                                        override_removals.insert(key.clone());
                                                    }
                                                    overrides_changed = true;
                                                }

                                                if enable_override {
                                                    let mut current_str = current_overrides
                                                        .get(&key)
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(default_str.as_str())
                                                        .to_string();
                                                    let before_str = current_str.clone();
                                                    egui::ComboBox::from_id_salt(format!("override_{comp_name}_{attr_name}"))
                                                    .selected_text(&current_str)
                                                    .show_ui(ui, |ui| {
                                                        for option in &attr_def.options {
                                                            ui.selectable_value(&mut current_str, option.clone(), option);
                                                        }
                                                    });
                                                    if current_str != before_str {
                                                        override_updates.insert(key, serde_json::Value::String(current_str));
                                                        overrides_changed = true;
                                                    }
                                                } else {
                                                    ui.label(egui::RichText::new(format!("Type default: {default_str}")).weak().italics());
                                                }
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            ui.add_space(4.0);
                        }
                    }
                    
                    changed |= draw_z_layer_legend(ui, &mut z);

                    if changed || overrides_changed {
                        crate::level::push_undo_snapshot(&mut undo_history, &document.level);
                        if let Some(entity) = document.level.entities.get_mut(index) {
                            entity.x = x;
                            entity.y = y;
                            entity.z_index = Some(z);
                            crate::editor::apply_flat_component_updates(entity, &override_removals, override_updates);
                        }
                        document.dirty = true;
                        scene_dirty.0 = true;
                    }
                }
            } else if selection.bounds_selected {
                ui.label("Level background / Bounds selected");
                ui.label("Origin: (0, 0) — fixed");
                ui.add_space(8.0);

                let bounds = document.level.bounds.get_or_insert(LevelBoundsDefinition { width: 1000.0, height: 1024.0 });
                let mut width = bounds.width;
                let mut height = bounds.height;
                let mut bounds_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    bounds_changed |= ui.add(egui::DragValue::new(&mut width).speed(1.0).range(1.0..=50000.0)).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    bounds_changed |= ui.add(egui::DragValue::new(&mut height).speed(1.0).range(1.0..=50000.0)).changed();
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

                let camera_center = crate::editor::camera_center_world(&camera_query, &window_query);
                let spawn_position = camera_center.unwrap_or(bevy::prelude::Vec2::ZERO);
                let mut add_requested: Option<String> = None;

                ui.push_id("add_menu_entity_types_scroll", |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("editor_add_menu_entity_types_scroll")
                        .show(ui, |ui| {
                        for entity_type_name in entity_type_names {
                            ui.push_id(format!("addmenu_entity_type:{}", entity_type_name), |ui| {
                                if ui.button(&entity_type_name).clicked() {
                                    add_requested = Some(entity_type_name);
                                }
                            });
                        }
                    });
                });

                if let Some(entity_type_name) = add_requested {
                    let is_player = document
                        .entity_types
                        .get(&entity_type_name)
                        .map(crate::editor::is_player_entity_type)
                        .unwrap_or(false);

                    let player_already_exists = is_player && document.level.entities.iter().any(|e| {
                        document
                            .entity_types
                            .get(&e.entity_type)
                            .map(crate::editor::is_player_entity_type)
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

    if *show_close_confirm {
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
                                *show_close_confirm = false;
                                next_state.set(crate::editor::EditorMode::LevelPicker);
                            }
                            Err(error) => {
                                toast.message = Some(format!("Save failed: {}", error));
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 4.0;
                            }
                        }
                    }

                    if ui.button("Discard and Close").clicked() {
                        document.dirty = false;
                        *show_close_confirm = false;
                        next_state.set(crate::editor::EditorMode::LevelPicker);
                    }

                    if ui.button("Cancel").clicked() {
                        *show_close_confirm = false;
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
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50)))
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
    egui::Grid::new("z_layer_legend_grid").num_columns(2).show(ui, |ui| {
        ui.label("Preset");
        ui.label("Z");
        ui.end_row();
        for (label, value, rgb) in [
            ("Foreground", 150.0_f32, [255u8, 0u8, 0u8]),
            ("Gameplay", 100.0_f32, [0u8, 255u8, 0u8]),
            ("Near Player", 50.0_f32, [255u8, 165u8, 0u8]),
            ("Background", 0.0_f32, [0u8, 0u8, 255u8]),
        ] {
            let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            ui.colored_label(color, label);
            ui.add(egui::DragValue::new(z).speed(1.0));
            ui.end_row();
        }
    });
    true
}
