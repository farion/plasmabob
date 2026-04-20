use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};
use egui::TextureId;
use egui_extras::{Column, TableBuilder};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use crate::entity_type::array_editor::{
    format_array_short, inner_array_value_to_csv_string, parse_array_type_signature,
    ArrayEditorState,
};
use crate::entity_type::array_property::render_array_property;
use crate::entity_type::bool_property::render_bool_property;
use crate::entity_type::components_sidebar::render_components_sidebar as sidebar_render_components_sidebar;
use crate::entity_type::enum_property::render_enum_property;
use crate::entity_type::helpers::{
    apply_to_staged_entity_type, cloned_staged_entity_type, component_default_value,
    component_object_snapshot, save_staged_entity_type, sorted_attribute_rows, AttributeUiRow,
};
use crate::entity_type::hitbox::{
    cursor_for_drag_edge, hitbox_to_screen, hitbox_to_screen_with_ratio, pick_hitbox_edge,
    units_per_pixel, ActiveHitboxDrag, DragEdge, EntityTypeEditorState, RectHitbox,
};
use crate::entity_type::json_property::render_json_property;
use crate::entity_type::number_property::render_number_property;
use crate::entity_type::string_property::render_string_property;

const HITBOX_EDGE_PICK_TOLERANCE_PX: f32 = 12.0;
const HITBOX_MIN_SIZE_PX: f32 = 1.0;
const PREVIEW_CANVAS_WIDTH_PX: f32 = 512.0;
const PREVIEW_CANVAS_HEIGHT_PX: f32 = 256.0;

/// Entity-Type detail view UI.
/// Shows: components, width/height and per-state animation images.
pub fn entity_type_view_ui(
    mut contexts: EguiContexts,
    view_state: Res<crate::level::state::EntityTypeViewState>,
    mut document: Option<ResMut<crate::level::state::EditorDocument>>,
    mut next_state: ResMut<NextState<crate::level::run::EditorMode>>,
    mut loaded_textures: Local<HashMap<String, TextureId>>,
    mut loaded_image_sizes: Local<HashMap<String, (u32, u32)>>,
    mut entity_type_editor: Local<EntityTypeEditorState>,
    mut show_close_confirm: Local<bool>,
    asset_server: Res<AssetServer>,
    active_character: Res<crate::level::run::ActiveCharacter>,
    mapping: Res<crate::level::state::ComponentValueMapping>,
    time: Res<Time>,
    mut toast: ResMut<crate::level::state::ToastState>,
    mut widths: ResMut<crate::core::ColumnWidths>,
) {
    // The body was moved verbatim from the previous entity_types.rs file.
    // Keeping the implementation here to minimize changes during the module
    // rename. See the original file for the full function body.
    // Note: After the move, submodules live under editor/src/entity_type/.

    // If nothing is selected, simply show a small message and return.
    if view_state.selected.is_none() {
        let Ok(ctx) = contexts.ctx_mut() else {
            return;
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("No entity type selected.");
        });
        return;
    }

    let selected_name = view_state.selected.clone().unwrap();

    if entity_type_editor.last_entity_type.as_deref() != Some(selected_name.as_str()) {
        entity_type_editor.edited_hitboxes.clear();
        entity_type_editor.dirty_states.clear();
        entity_type_editor.active_drag = None;
        entity_type_editor.add_selected = None;
        // Do not clear edited_entity_types here; keep staged edits across selections
        entity_type_editor.last_entity_type = Some(selected_name.clone());
    }

    // Try to obtain the EntityTypeDefinition from the loaded EditorDocument
    // if present. Otherwise attempt to read the JSON file directly from
    // assets/entity_types/<selected>.json so the dashboard click works without
    // opening a level.
    let et_data: Cow<'_, crate::core::EntityTypeDefinition> = if let Some(doc) = document.as_ref() {
        if let Some(et) = doc.entity_types.get(&selected_name) {
            // Clone the in-memory entity-type so we can safely mutate the
            // document later without conflicting borrows.
            Cow::Owned(et.clone())
        } else {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Entity type not found in loaded document list.");
            });
            return;
        }
    } else {
        // No document loaded: try to read from assets/entity_types/<selected>.json
        let assets_dir = crate::core::io::assets_dir();
        let json_path = assets_dir
            .join("entity_types")
            .join(format!("{}.json", selected_name));
        if !json_path.exists() {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!(
                    "Entity type JSON not found: {}",
                    json_path.display()
                ));
            });
            return;
        }

        let content = match std::fs::read_to_string(&json_path) {
            Ok(c) => c,
            Err(e) => {
                let Ok(ctx) = contexts.ctx_mut() else {
                    return;
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!("Error reading {}: {}", json_path.display(), e));
                });
                return;
            }
        };

        let parsed: Result<crate::core::EntityTypeDefinition, _> = serde_json::from_str(&content);
        let parsed = match parsed {
            Ok(p) => p,
            Err(e) => {
                let Ok(ctx) = contexts.ctx_mut() else {
                    return;
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!("Error parsing {}: {}", json_path.display(), e));
                });
                return;
            }
        };

        // Basic validation similar to io::validate_entity_type_definition
        let Some(state_machine) = parsed.state_machine() else {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!(
                    "Entity type '{}' requires 'components.state_machine'",
                    selected_name
                ));
            });
            return;
        };
        if state_machine.states.is_empty()
            || !state_machine
                .states
                .contains_key(&state_machine.initial_state)
        {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!(
                    "Entity type '{}' requires a non-empty 'components.state_machine.states' object containing initial_state '{}'",
                    selected_name,
                    state_machine.initial_state
                ));
            });
            return;
        }

        // Keep an owned parsed value and also store a staged editable copy so
        // component edits are staged in-memory and saved only on Ctrl+S.
        let parsed_owned = parsed;
        entity_type_editor
            .edited_entity_types
            .entry(selected_name.clone())
            .or_insert_with(|| parsed_owned.clone());

        Cow::Owned(parsed_owned)
    };
    let et_ref = et_data.as_ref();
    let Some(state_machine) = et_ref.state_machine() else {
        let Ok(ctx) = contexts.ctx_mut() else {
            return;
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "Entity type '{}' requires 'components.state_machine'",
                selected_name
            ));
        });
        return;
    };

    // Preload/ensure TextureIds for all animation frames so we don't need to
    // call `contexts.add_image()` while holding an `egui::Context` borrow.
    let mut all_paths: Vec<String> = Vec::new();
    for state in state_machine.states.values() {
        for path in &state.animation {
            all_paths.push(crate::core::normalize_asset_reference(path));
        }
    }
    all_paths.sort();
    all_paths.dedup();

    for normalized in all_paths.iter() {
        if !loaded_textures.contains_key(normalized) {
            // Resolve using active character: if original file missing, try suffixed variant.
            let resolved = {
                let fs_exact = crate::core::io::assets_dir().join(normalized);
                if fs_exact.exists() {
                    normalized.clone()
                } else if let Some(pos) = normalized.rfind('.') {
                    let (before_ext, ext) = normalized.split_at(pos);
                    if before_ext.ends_with(".bob") || before_ext.ends_with(".betty") {
                        normalized.clone()
                    } else {
                        let suf = match *active_character {
                            crate::level::run::ActiveCharacter::Betty => "betty",
                            _ => "bob",
                        };
                        let suffixed = format!("{}.{suf}{}", before_ext, ext);
                        let fs_suff = crate::core::io::assets_dir().join(&suffixed);
                        if fs_suff.exists() {
                            suffixed
                        } else {
                            normalized.clone()
                        }
                    }
                } else {
                    normalized.clone()
                }
            };

            let handle: Handle<Image> = asset_server.load(resolved.clone());
            let tex_id = contexts.add_image(EguiTextureHandle::Strong(handle));
            loaded_textures.insert(normalized.clone(), tex_id);

            // Attempt to read image dimensions from the resolved filesystem path
            // (prefer suffixed file when that was selected). Store dimensions
            // keyed by the normalized asset key so the rest of the UI uses the
            // same lookup key.
            if !loaded_image_sizes.contains_key(normalized) {
                let fs_resolved = crate::core::io::asset_path_to_filesystem_path(&resolved);
                if let Ok((w, h)) = image::image_dimensions(&fs_resolved) {
                    loaded_image_sizes.insert(normalized.clone(), (w, h));
                }
            }
        }

        if !loaded_image_sizes.contains_key(normalized) {
            // Resolve filesystem path and attempt to read dimensions. This is
            // best-effort: failure to read dimensions is non-fatal and will
            // fall back to the square preview size.
            let fs_path = crate::core::io::asset_path_to_filesystem_path(normalized);
            if let Ok((w, h)) = image::image_dimensions(&fs_path) {
                loaded_image_sizes.insert(normalized.clone(), (w, h));
            }
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    // Determine dirty mark for the selected entity type and show it in the top title
    let dirty_mark = if entity_type_editor
        .dirty_entity_types
        .contains(&selected_name)
    {
        " *"
    } else {
        ""
    };

    // Build a normalized asset-like path for the entity type so the title matches
    // the Level editor style (e.g. "entity_types/cockroach.json"). The
    // `selected_name` may already include the ".json" suffix; handle both cases.
    let full_asset_path = if selected_name.to_lowercase().ends_with(".json") {
        format!("entity_types/{}", selected_name)
    } else {
        format!("entity_types/{}.json", selected_name)
    };

    // Top bar with Close button (right aligned). Title shows the asset-like path
    // and the dirty '*' when applicable.
    egui::TopBottomPanel::top("entity_type_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(format!("{}{}", full_asset_path, dirty_mark));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui_phosphor_icons::icons::X).clicked() {
                    // If dirty, show confirm dialog; otherwise close immediately.
                    if entity_type_editor
                        .dirty_entity_types
                        .contains(&selected_name)
                    {
                        *show_close_confirm = true;
                    } else {
                        next_state.set(crate::level::run::EditorMode::LevelPicker);
                    }
                }
            });
        });
    });

    if !ctx.input(|input| input.pointer.primary_down()) {
        entity_type_editor.active_drag = None;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::L)) {
        entity_type_editor.show_hitboxes = !entity_type_editor.show_hitboxes;
    }

    if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
        // Save staged component and hitbox changes for the currently selected entity type.
        if !entity_type_editor
            .dirty_entity_types
            .contains(&selected_name)
            && entity_type_editor.dirty_states.is_empty()
        {
            // no-op when nothing to save
        } else {
            match save_staged_entity_type(
                document.as_ref().map(|doc| doc.as_ref()),
                &entity_type_editor,
                &selected_name,
                et_ref,
            ) {
                Ok(()) => {
                    entity_type_editor.dirty_states.clear();
                    entity_type_editor.dirty_entity_types.remove(&selected_name);
                }
                Err(_error) => {
                    // ignore save error here (status messages removed)
                }
            }
        }
    }

    sidebar_render_components_sidebar(
        ctx,
        &selected_name,
        &mut document,
        &mut entity_type_editor,
        et_ref,
        &mapping,
        &mut toast,
        &time,
        // pass ColumnWidths resource so the sidebar can update shared widths
        widths,
    );

    egui::CentralPanel::default().show(ctx, |ui| {
        // Make the entire central content scrollable in both directions so the
        // whole sprite area can be scrolled horizontally when necessary.
        egui::ScrollArea::both()
            .id_salt(format!("entity_type_main_scroll_{}", selected_name))
            .show(ui, |ui| {
                ui.add_space(6.0);

                ui.label("L: toggle hitboxes | Drag edge: adjust hitbox | Ctrl+S: save");

                // The selected entity type is shown in the header; no separate
                // "Selected:" label is needed here.
                ui.separator();

                // Size
                ui.horizontal(|ui| {
                    ui.label(format!("Width: {} px", et_ref.width.unwrap_or_default()));
                    ui.add_space(12.0);
                    ui.label(format!("Height: {} px", et_ref.height.unwrap_or_default()));
                });

                ui.add_space(8.0);

                // States and images
                ui.label(egui::RichText::new("States / Animation Frames").strong());
                ui.add_space(4.0);

                // Sort states for stable order
                let mut state_keys: Vec<_> = state_machine.states.keys().cloned().collect();
                state_keys.sort();

                for state_key in state_keys {
                    if let Some(state_def) = state_machine.states.get(&state_key) {
                        egui::CollapsingHeader::new(state_key.clone())
                            .id_salt(format!(
                                "entity_type_state_header_{}_{}",
                                selected_name, state_key
                            ))
                            .default_open(true)
                            .show(ui, |ui| {
                                if state_def.animation.is_empty() {
                                    ui.label("(no animation / no images)");
                                    return;
                                }

                                // Show frames in a single horizontal row. The outer
                                // ScrollArea::both() provides horizontal scrolling for
                                // the entire area, so we only need a simple horizontal
                                // layout here.
                                ui.horizontal(|ui| {
                                    for frame_path in &state_def.animation {
                                        let normalized =
                                            crate::core::normalize_asset_reference(frame_path);

                                        // Ensure we have a cached TextureId for this asset path. We
                                        // preloaded textures above, so avoid calling
                                        // `contexts.add_image()` here to prevent multiple mutable
                                        // borrows. If missing, show a placeholder label.
                                        let texture =
                                            if let Some(&th) = loaded_textures.get(&normalized) {
                                                th
                                            } else {
                                                ui.vertical(|ui| {
                                                    ui.label("(preview not loaded)");
                                                    ui.label(normalized.clone());
                                                });
                                                continue;
                                            };

                                        // Compute aspect-preserving display size fitting into a 512x256 canvas.
                                        let display_size = if let Some((w, h)) =
                                            loaded_image_sizes.get(&normalized)
                                        {
                                            let w_f = *w as f32;
                                            let h_f = *h as f32;
                                            if w_f > 0.0 && h_f > 0.0 {
                                                let scale = (PREVIEW_CANVAS_WIDTH_PX / w_f)
                                                    .min(PREVIEW_CANVAS_HEIGHT_PX / h_f);
                                                egui::vec2(w_f * scale, h_f * scale)
                                            } else {
                                                egui::vec2(
                                                    PREVIEW_CANVAS_WIDTH_PX,
                                                    PREVIEW_CANVAS_HEIGHT_PX,
                                                )
                                            }
                                        } else {
                                            egui::vec2(
                                                PREVIEW_CANVAS_WIDTH_PX,
                                                PREVIEW_CANVAS_HEIGHT_PX,
                                            )
                                        };

                                        // Display small preview + path label
                                        ui.vertical(|ui| {
                                            // Use the image's display size as the frame size so the
                                            // drawn border exactly matches the sprite preview size.
                                            // Fall back to the preview canvas when dimensions are
                                            // unavailable (best-effort).
                                            let frame_size = display_size;
                                            let (canvas_rect, mut response) = ui
                                                .allocate_exact_size(
                                                    frame_size,
                                                    egui::Sense::click_and_drag(),
                                                );
                                            // When the frame equals the image display size the image
                                            // rect is the same as the canvas rect which avoids any
                                            // apparent padding between border and sprite.
                                            let image_rect = canvas_rect;

                                            // Draw a 1px subtle border exactly around the image/frame.
                                            ui.painter().rect_stroke(
                                                canvas_rect,
                                                0.0,
                                                egui::Stroke::new(
                                                    1.0,
                                                    egui::Color32::from_gray(70),
                                                ),
                                                egui::StrokeKind::Inside,
                                            );
                                            ui.painter().image(
                                                texture,
                                                image_rect,
                                                egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                ),
                                                egui::Color32::WHITE,
                                            );

                                            if entity_type_editor.show_hitboxes {
                                                let image_dims = loaded_image_sizes
                                                    .get(&normalized)
                                                    .map(|(w, h)| egui::vec2(*w as f32, *h as f32))
                                                    .unwrap_or_else(|| {
                                                        egui::vec2(display_size.x, display_size.y)
                                                    });
                                                let ratio_units_per_pixel = units_per_pixel(
                                                    et_ref.height.unwrap_or_default(),
                                                    image_dims.y,
                                                );

                                                let mut rect = entity_type_editor
                                                    .edited_hitboxes
                                                    .get(&state_key)
                                                    .copied()
                                                    .unwrap_or_else(|| {
                                                        RectHitbox::from_points(
                                                            state_def.hitbox_points(),
                                                            image_dims.x,
                                                            image_dims.y,
                                                            ratio_units_per_pixel,
                                                        )
                                                    });

                                                rect.clamp_to_image(
                                                    image_dims.x,
                                                    image_dims.y,
                                                    ratio_units_per_pixel,
                                                );
                                                let screen_hitbox = hitbox_to_screen_with_ratio(
                                                    rect,
                                                    image_rect,
                                                    image_dims,
                                                    ratio_units_per_pixel,
                                                );

                                                if let Some(pointer_pos) = response.hover_pos() {
                                                    if let Some(edge) =
                                                        pick_hitbox_edge(pointer_pos, screen_hitbox)
                                                    {
                                                        response = response.on_hover_cursor(
                                                            cursor_for_drag_edge(edge),
                                                        );
                                                    }
                                                }

                                                ui.painter().rect_stroke(
                                                    screen_hitbox,
                                                    0.0,
                                                    egui::Stroke::new(2.0, egui::Color32::RED),
                                                    egui::StrokeKind::Inside,
                                                );

                                                if response.drag_started() {
                                                    if let Some(pointer_pos) =
                                                        response.interact_pointer_pos()
                                                    {
                                                        if let Some(edge) = pick_hitbox_edge(
                                                            pointer_pos,
                                                            screen_hitbox,
                                                        ) {
                                                            entity_type_editor.active_drag =
                                                                Some(ActiveHitboxDrag {
                                                                    state_key: state_key.clone(),
                                                                    edge,
                                                                });
                                                        }
                                                    }
                                                }

                                                if response.dragged() {
                                                    if let Some(active_drag) =
                                                        &entity_type_editor.active_drag
                                                    {
                                                        if active_drag.state_key == state_key {
                                                            let pointer_delta =
                                                                ui.ctx().input(|input| {
                                                                    input.pointer.delta()
                                                                });
                                                            let x_scale = if image_dims.x > 0.0 {
                                                                image_rect.width() / image_dims.x
                                                            } else {
                                                                1.0
                                                            };
                                                            let y_scale = if image_dims.y > 0.0 {
                                                                image_rect.height() / image_dims.y
                                                            } else {
                                                                1.0
                                                            };
                                                            let image_delta = egui::vec2(
                                                                pointer_delta.x / x_scale,
                                                                -pointer_delta.y / y_scale,
                                                            );
                                                            let units_delta = egui::vec2(
                                                                image_delta.x
                                                                    * ratio_units_per_pixel,
                                                                image_delta.y
                                                                    * ratio_units_per_pixel,
                                                            );

                                                            rect.drag_edge(
                                                                active_drag.edge,
                                                                units_delta,
                                                                image_dims.x,
                                                                image_dims.y,
                                                                ratio_units_per_pixel,
                                                            );
                                                            entity_type_editor
                                                                .dirty_states
                                                                .insert(state_key.clone());
                                                            // Mark the whole entity type as dirty so the title shows '*' and
                                                            // Ctrl+S will save staged changes.
                                                            entity_type_editor
                                                                .dirty_entity_types
                                                                .insert(selected_name.clone());
                                                        }
                                                    }
                                                }

                                                entity_type_editor
                                                    .edited_hitboxes
                                                    .insert(state_key.clone(), rect);
                                            }

                                            ui.label(normalized.clone());
                                        });
                                    }
                                });
                            });
                    }
                }
            });
    });

    if let Some(component_name) = entity_type_editor.remove_component_confirm.clone() {
        egui::Window::new("Confirm Remove")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Remove component '{}' from entity type '{}' ?",
                    component_name, selected_name
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Remove").clicked() {
                        let snapshot = cloned_staged_entity_type(
                            document.as_deref(),
                            &entity_type_editor,
                            &selected_name,
                            et_ref,
                        );
                        let mut new_components = snapshot.component_names();
                        new_components.retain(|name| name != &component_name);

                        if apply_to_staged_entity_type(
                            document.as_deref_mut(),
                            &mut entity_type_editor,
                            &selected_name,
                            et_ref,
                            |et| et.set_component_names(&new_components),
                        ) {
                            entity_type_editor
                                .dirty_entity_types
                                .insert(selected_name.clone());
                        }
                        entity_type_editor.remove_component_confirm = None;
                    }

                    if ui.button("Cancel").clicked() {
                        entity_type_editor.remove_component_confirm = None;
                    }
                });
            });
    }

    // Confirm close dialog when requested
    if *show_close_confirm {
        egui::Window::new("Confirm Close")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("There are unsaved changes for this entity type.");
                ui.label("Save before closing?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save and Close").clicked() {
                        match save_staged_entity_type(
                            document.as_ref().map(|doc| doc.as_ref()),
                            &entity_type_editor,
                            &selected_name,
                            et_ref,
                        ) {
                            Ok(()) => {
                                entity_type_editor.dirty_states.clear();
                                entity_type_editor.dirty_entity_types.remove(&selected_name);
                                *show_close_confirm = false;
                                next_state.set(crate::level::run::EditorMode::LevelPicker);
                            }
                            Err(_e) => {
                                // Show a toast on failure
                                toast.message = Some("Save failed while closing".to_string());
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                            }
                        }
                    }

                    if ui.button("Discard and Close").clicked() {
                        // Drop staged edits for this entity type and close
                        entity_type_editor
                            .edited_entity_types
                            .remove(&selected_name);
                        entity_type_editor.dirty_entity_types.remove(&selected_name);
                        entity_type_editor.dirty_states.clear();
                        *show_close_confirm = false;
                        next_state.set(crate::level::run::EditorMode::LevelPicker);
                    }

                    if ui.button("Cancel").clicked() {
                        *show_close_confirm = false;
                    }
                });
            });
    }
}
