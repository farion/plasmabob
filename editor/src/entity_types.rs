use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};
use egui::TextureId;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

const HITBOX_EDGE_PICK_TOLERANCE_PX: f32 = 12.0;
const HITBOX_MIN_SIZE_PX: f32 = 1.0;
const PREVIEW_CANVAS_WIDTH_PX: f32 = 512.0;
const PREVIEW_CANVAS_HEIGHT_PX: f32 = 256.0;

#[derive(Clone, Copy, Debug)]
enum DragEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug)]
struct ActiveHitboxDrag {
    state_key: String,
    edge: DragEdge,
}

#[derive(Clone, Copy, Debug)]
struct RectHitbox {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl RectHitbox {
    fn from_points(points: &[[f32; 2]], image_w: f32, image_h: f32, units_per_pixel: f32) -> Self {
        let max_w_units = image_w.max(HITBOX_MIN_SIZE_PX) * units_per_pixel;
        let max_h_units = image_h.max(HITBOX_MIN_SIZE_PX) * units_per_pixel;

        if points.is_empty() {
            return Self {
                left: 0.0,
                right: max_w_units,
                bottom: 0.0,
                top: max_h_units,
            };
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for [x, y] in points {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }

        let mut rect = Self {
            left: min_x,
            right: max_x,
            bottom: min_y,
            top: max_y,
        };
        rect.clamp_to_image(image_w, image_h, units_per_pixel);
        rect
    }

    fn clamp_to_image(&mut self, image_w: f32, image_h: f32, units_per_pixel: f32) {
        let min_size_units = HITBOX_MIN_SIZE_PX * units_per_pixel;
        let max_w = image_w.max(HITBOX_MIN_SIZE_PX) * units_per_pixel;
        let max_h = image_h.max(HITBOX_MIN_SIZE_PX) * units_per_pixel;

        self.left = self.left.clamp(0.0, max_w - min_size_units);
        self.right = self.right.clamp(self.left + min_size_units, max_w);
        self.bottom = self.bottom.clamp(0.0, max_h - min_size_units);
        self.top = self.top.clamp(self.bottom + min_size_units, max_h);
    }

    fn drag_edge(&mut self, edge: DragEdge, units_delta: egui::Vec2, image_w: f32, image_h: f32, units_per_pixel: f32) {
        match edge {
            DragEdge::Left => {
                self.left += units_delta.x;
            }
            DragEdge::Right => {
                self.right += units_delta.x;
            }
            DragEdge::Bottom => {
                self.bottom += units_delta.y;
            }
            DragEdge::Top => {
                self.top += units_delta.y;
            }
        }
        self.clamp_to_image(image_w, image_h, units_per_pixel);
    }

    fn to_json_points(self) -> [[f32; 2]; 4] {
        [
            [self.left, self.bottom],
            [self.right, self.bottom],
            [self.right, self.top],
            [self.left, self.top],
        ]
    }
}

pub(crate) struct HitboxEditorState {
    show_hitboxes: bool,
    active_drag: Option<ActiveHitboxDrag>,
    edited_hitboxes: HashMap<String, RectHitbox>,
    dirty_states: HashSet<String>,
    last_entity_type: Option<String>,
        add_selected: Option<String>,
    dirty_entity_types: HashSet<String>,
    edited_entity_types: HashMap<String, crate::model::EntityTypeDefinition>,
}

impl Default for HitboxEditorState {
    fn default() -> Self {
        Self {
            // Enable hitbox overlays by default per user request
            show_hitboxes: true,
            active_drag: None,
            edited_hitboxes: HashMap::new(),
            dirty_states: HashSet::new(),
            last_entity_type: None,
            add_selected: None,
            dirty_entity_types: HashSet::new(),
            edited_entity_types: HashMap::new(),
        }
    }
}

fn hitbox_to_screen(rect: RectHitbox, image_rect: egui::Rect, image_size: egui::Vec2) -> egui::Rect {
    let x_scale = if image_size.x > 0.0 {
        image_rect.width() / image_size.x
    } else {
        1.0
    };
    let y_scale = if image_size.y > 0.0 {
        image_rect.height() / image_size.y
    } else {
        1.0
    };

    let left = image_rect.left() + rect.left * x_scale;
    let right = image_rect.left() + rect.right * x_scale;
    let bottom = image_rect.bottom() - rect.bottom * y_scale;
    let top = image_rect.bottom() - rect.top * y_scale;

    egui::Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom))
}

fn units_per_pixel(configured_height: f32, image_height: f32) -> f32 {
    if configured_height > 0.0 && image_height > 0.0 {
        configured_height / image_height
    } else {
        1.0
    }
}

fn hitbox_to_screen_with_ratio(
    rect_units: RectHitbox,
    image_rect: egui::Rect,
    image_size_pixels: egui::Vec2,
    units_per_pixel_value: f32,
) -> egui::Rect {
    let inv = if units_per_pixel_value > 0.0 {
        1.0 / units_per_pixel_value
    } else {
        1.0
    };
    let rect_pixels = RectHitbox {
        left: rect_units.left * inv,
        right: rect_units.right * inv,
        bottom: rect_units.bottom * inv,
        top: rect_units.top * inv,
    };

    hitbox_to_screen(rect_pixels, image_rect, image_size_pixels)
}

fn pick_hitbox_edge(pointer: egui::Pos2, screen_hitbox: egui::Rect) -> Option<DragEdge> {
    if !screen_hitbox.expand(HITBOX_EDGE_PICK_TOLERANCE_PX).contains(pointer) {
        return None;
    }

    let mut candidates = [
        (DragEdge::Left, (pointer.x - screen_hitbox.left()).abs()),
        (DragEdge::Right, (pointer.x - screen_hitbox.right()).abs()),
        (DragEdge::Bottom, (pointer.y - screen_hitbox.bottom()).abs()),
        (DragEdge::Top, (pointer.y - screen_hitbox.top()).abs()),
    ];
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

    if candidates[0].1 <= HITBOX_EDGE_PICK_TOLERANCE_PX {
        Some(candidates[0].0)
    } else {
        None
    }
}

fn cursor_for_drag_edge(edge: DragEdge) -> egui::CursorIcon {
    match edge {
        DragEdge::Left | DragEdge::Right => egui::CursorIcon::ResizeHorizontal,
        DragEdge::Top | DragEdge::Bottom => egui::CursorIcon::ResizeVertical,
    }
}

/// Entity-Type detail view UI.
/// Shows: components, width/height and per-state animation images.
pub(crate) fn entity_type_view_ui(
    mut contexts: EguiContexts,
    view_state: Res<crate::editor::EntityTypeViewState>,
    mut document: Option<ResMut<crate::editor::EditorDocument>>,
    mut next_state: ResMut<NextState<crate::editor::EditorMode>>,
    mut loaded_textures: Local<HashMap<String, TextureId>>,
    mut loaded_image_sizes: Local<HashMap<String, (u32, u32)>>,
    mut hitbox_editor: Local<HitboxEditorState>,
    mut show_close_confirm: Local<bool>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut toast: ResMut<crate::editor::ToastState>,
) {
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

    if hitbox_editor.last_entity_type.as_deref() != Some(selected_name.as_str()) {
        hitbox_editor.edited_hitboxes.clear();
        hitbox_editor.dirty_states.clear();
        hitbox_editor.active_drag = None;
        hitbox_editor.add_selected = None;
        // Do not clear edited_entity_types here; keep staged edits across selections
        hitbox_editor.last_entity_type = Some(selected_name.clone());
    }

    // Try to obtain the EntityTypeDefinition from the loaded EditorDocument
    // if present. Otherwise attempt to read the JSON file directly from
    // assets/entity_types/<selected>.json so the dashboard click works without
    // opening a level.
    let et_data: Cow<'_, crate::model::EntityTypeDefinition> = if let Some(doc) = document.as_ref() {
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
        let assets_dir = crate::io::assets_dir();
        let json_path = assets_dir.join("entity_types").join(format!("{}.json", selected_name));
        if !json_path.exists() {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!("Entity type JSON not found: {}", json_path.display()));
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

        let parsed: Result<crate::model::EntityTypeDefinition, _> = serde_json::from_str(&content);
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
        if parsed.states.is_empty() || !parsed.states.contains_key("default") {
            let Ok(ctx) = contexts.ctx_mut() else {
                return;
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!(
                    "Entity type '{}' requires a non-empty 'states' object with 'default'",
                    selected_name
                ));
            });
            return;
        }

        // Keep an owned parsed value and also store a staged editable copy so
        // component edits are staged in-memory and saved only on Ctrl+S.
        let parsed_owned = parsed;
        hitbox_editor
            .edited_entity_types
            .entry(selected_name.clone())
            .or_insert_with(|| parsed_owned.clone());

        Cow::Owned(parsed_owned)
    };
    let et_ref = et_data.as_ref();

    // Preload/ensure TextureIds for all animation frames so we don't need to
    // call `contexts.add_image()` while holding an `egui::Context` borrow.
    let mut all_paths: Vec<String> = Vec::new();
    for state in et_ref.states.values() {
        for path in &state.animation {
            all_paths.push(crate::model::normalize_asset_reference(path));
        }
    }
    all_paths.sort();
    all_paths.dedup();

    for normalized in all_paths.iter() {
        if !loaded_textures.contains_key(normalized) {
            let handle: Handle<Image> = asset_server.load(normalized);
            let tex_id = contexts.add_image(EguiTextureHandle::Strong(handle));
            loaded_textures.insert(normalized.clone(), tex_id);
        }

        if !loaded_image_sizes.contains_key(normalized) {
            // Resolve filesystem path and attempt to read dimensions. This is
            // best-effort: failure to read dimensions is non-fatal and will
            // fall back to the square preview size.
            let fs_path = crate::io::asset_path_to_filesystem_path(normalized);
            if let Ok((w, h)) = image::image_dimensions(&fs_path) {
                loaded_image_sizes.insert(normalized.clone(), (w, h));
            }
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    // Determine dirty mark for the selected entity type and show it in the top title
    let dirty_mark = if hitbox_editor.dirty_entity_types.contains(&selected_name) { " *" } else { "" };

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
                if ui.button("Close").clicked() {
                    // If dirty, show confirm dialog; otherwise close immediately.
                    if hitbox_editor.dirty_entity_types.contains(&selected_name) {
                        *show_close_confirm = true;
                    } else {
                        next_state.set(crate::editor::EditorMode::LevelPicker);
                    }
                }
            });
        });
    });

    if !ctx.input(|input| input.pointer.primary_down()) {
        hitbox_editor.active_drag = None;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::L)) {
        hitbox_editor.show_hitboxes = !hitbox_editor.show_hitboxes;
    }

    if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
        // Save staged component and hitbox changes for the currently selected entity type.
        if !hitbox_editor.dirty_entity_types.contains(&selected_name) && hitbox_editor.dirty_states.is_empty() {
            // no-op when nothing to save
        } else {
            // Prepare components to save: prefer staged copy, then document copy
            let components_to_save: Option<Vec<String>> = if let Some(doc_ref) = document.as_ref() {
                doc_ref.entity_types.get(&selected_name).map(|et| et.components.clone())
            } else {
                hitbox_editor
                    .edited_entity_types
                    .get(&selected_name)
                    .map(|et| et.components.clone())
            };

            let comp_result = if let Some(components) = components_to_save {
                crate::io::save_entity_type_components(&selected_name, &components)
            } else {
                Ok(())
            };

            // Prepare hitbox save map
            let mut save_map: HashMap<String, [[f32; 2]; 4]> = HashMap::new();
            for state_key in &hitbox_editor.dirty_states {
                if let Some(rect) = hitbox_editor.edited_hitboxes.get(state_key) {
                    save_map.insert(state_key.clone(), rect.to_json_points());
                }
            }

            let hitbox_result = if save_map.is_empty() {
                Ok(())
            } else {
                crate::io::save_entity_type_hitboxes(&selected_name, &save_map)
            };

            match comp_result.and(hitbox_result) {
                Ok(()) => {
                    hitbox_editor.dirty_states.clear();
                    hitbox_editor.dirty_entity_types.remove(&selected_name);
                }
                Err(_error) => {
                    // ignore save error here (status messages removed)
                }
            }
        }
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        // Make the entire central content scrollable
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);

            ui.label("L: toggle hitboxes | Drag edge: adjust hitbox | Ctrl+S: save");

            // The selected entity type is shown in the header; no separate
            // "Selected:" label is needed here.
            ui.separator();

            // Components (editable)
            ui.group(|ui| {
                ui.label(egui::RichText::new("Components").strong());
                ui.add_space(4.0);

                // Scan available gameplay components from src/game/components
                let available_components = match crate::io::scan_game_components() {
                    Ok(v) => v,
                    Err(e) => {
                        // Show a temporary toast when scanning fails
                        toast.message = Some(format!("Could not scan components: {}", e));
                        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                        Vec::new()
                    }
                };

                // Compute current components snapshot (prefer live document if present)
                let components_snapshot: Vec<String> = if let Some(doc_ref) = document.as_ref() {
                    doc_ref
                        .entity_types
                        .get(&selected_name)
                        .map(|et| et.components.clone())
                        .unwrap_or_else(|| et_ref.components.clone())
                } else {
                    hitbox_editor
                        .edited_entity_types
                        .get(&selected_name)
                        .map(|et| et.components.clone())
                        .unwrap_or_else(|| et_ref.components.clone())
                };

                // Show components inline with a small 'X' button to remove
                ui.horizontal_wrapped(|ui| {
                    for comp in &components_snapshot {
                        ui.horizontal(|ui| {
                            ui.label(format!("[{}]", comp));
                                if ui.small_button("X").clicked() {
                                let mut new_components = components_snapshot.clone();
                                new_components.retain(|c| c != comp);

                                if let Some(doc_mut) = document.as_mut() {
                                        if let Some(et) = doc_mut.entity_types.get_mut(&selected_name) {
                                            // Stage component change in-memory. Do not write file yet.
                                            et.components = new_components.clone();
                                        } else {
                                            // entity type not found in document; no status message
                                        }
                                } else {
                                    // Update staged copy for unloaded document mode
                                    hitbox_editor
                                        .edited_entity_types
                                        .entry(selected_name.clone())
                                        .or_insert_with(|| et_ref.clone())
                                        .components = new_components.clone();
                                }
                                // mark entity type dirty for saving via Ctrl+S
                                hitbox_editor.dirty_entity_types.insert(selected_name.clone());
                            }
                        });
                        ui.add_space(6.0);
                    }
                });

                ui.separator();
                ui.add_space(4.0);

                // Add component pulldown (ComboBox) — only show components not already present
                let add_options: Vec<String> = available_components
                    .into_iter()
                    .filter(|s| !components_snapshot.iter().any(|c| c == s))
                    .collect();

                if !add_options.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Add component:");

                        // Persist previous selection and provide a local mutable copy for the ComboBox
                        let prev_sel = hitbox_editor.add_selected.clone().unwrap_or_default();
                        let mut add_sel = prev_sel.clone();

                        egui::ComboBox::from_id_salt(format!("add_component_cb_{}", selected_name))
                            .selected_text(if add_sel.is_empty() { "select..." } else { &add_sel })
                            .show_ui(ui, |ui| {
                                        for opt in &add_options {
                                                    ui.selectable_value(&mut add_sel, opt.clone(), opt);
                                                }
                            });

                        // If selection changed to a non-empty value, apply it
                        if !add_sel.is_empty() && add_sel != prev_sel {
                            let chosen = add_sel.clone();
                            let mut new_components = components_snapshot.clone();
                                if !new_components.iter().any(|c| c == &chosen) {
                                    new_components.push(chosen.clone());
                                    if let Some(doc_mut) = document.as_mut() {
                                        if let Some(et) = doc_mut.entity_types.get_mut(&selected_name) {
                                            // Stage in-memory change; do not write to disk yet
                                            et.components = new_components.clone();
                                            // staged
                                        } else {
                                            // entity type not found in document
                                        }
                                    } else {
                                        hitbox_editor
                                            .edited_entity_types
                                            .entry(selected_name.clone())
                                            .or_insert_with(|| et_ref.clone())
                                            .components = new_components.clone();
                                        // staged
                                    }
                                }
                                // mark entity type dirty for saving via Ctrl+S
                                hitbox_editor.dirty_entity_types.insert(selected_name.clone());
                            // clear stored selection
                            hitbox_editor.add_selected = None;
                        } else {
                            // persist current selection into state (or None if empty)
                            hitbox_editor.add_selected = if add_sel.is_empty() { None } else { Some(add_sel) };
                        }
                    });
                } else {
                    ui.label("(no additional components available)");
                }
            });

            ui.add_space(6.0);

            // Size
            ui.horizontal(|ui| {
                ui.label(format!("Width: {} px", et_ref.width));
                ui.add_space(12.0);
                ui.label(format!("Height: {} px", et_ref.height));
            });

            ui.add_space(8.0);

            // States and images
            ui.label(egui::RichText::new("States / Animation Frames").strong());
            ui.add_space(4.0);

            // Sort states for stable order
            let mut state_keys: Vec<_> = et_ref.states.keys().cloned().collect();
            state_keys.sort();

            for state_key in state_keys {
                if let Some(state_def) = et_ref.states.get(&state_key) {
                    egui::CollapsingHeader::new(state_key.clone()).default_open(true).show(ui, |ui| {
                        if state_def.animation.is_empty() {
                            ui.label("(no animation / no images)");
                            return;
                        }

                        // Show frames in a grid-like horizontal flow
                        ui.horizontal_wrapped(|ui| {
                            for frame_path in &state_def.animation {
                                let normalized = crate::model::normalize_asset_reference(frame_path);

                                // Ensure we have a cached TextureId for this asset path. We
                                // preloaded textures above, so avoid calling
                                // `contexts.add_image()` here to prevent multiple mutable
                                // borrows. If missing, show a placeholder label.
                                let texture = if let Some(&th) = loaded_textures.get(&normalized) {
                                    th
                                    } else {
                                    ui.vertical(|ui| {
                                        ui.label("(preview not loaded)");
                                        ui.label(normalized.clone());
                                    });
                                    continue;
                                };

                                // Compute aspect-preserving display size fitting into a 512x256 canvas.
                                let display_size = if let Some((w, h)) = loaded_image_sizes.get(&normalized) {
                                    let w_f = *w as f32;
                                    let h_f = *h as f32;
                                    if w_f > 0.0 && h_f > 0.0 {
                                        let scale = (PREVIEW_CANVAS_WIDTH_PX / w_f)
                                            .min(PREVIEW_CANVAS_HEIGHT_PX / h_f);
                                        egui::vec2(w_f * scale, h_f * scale)
                                    } else {
                                        egui::vec2(PREVIEW_CANVAS_WIDTH_PX, PREVIEW_CANVAS_HEIGHT_PX)
                                    }
                                } else {
                                    egui::vec2(PREVIEW_CANVAS_WIDTH_PX, PREVIEW_CANVAS_HEIGHT_PX)
                                };

                                // Display small preview + path label
                                ui.vertical(|ui| {
                                    let canvas_size = egui::vec2(PREVIEW_CANVAS_WIDTH_PX, PREVIEW_CANVAS_HEIGHT_PX);
                                    let (canvas_rect, mut response) = ui.allocate_exact_size(
                                        canvas_size,
                                        egui::Sense::click_and_drag(),
                                    );
                                    let image_rect = egui::Rect::from_center_size(canvas_rect.center(), display_size);

                                    ui.painter().rect_stroke(
                                        canvas_rect,
                                        0.0,
                                        egui::Stroke::new(1.0, egui::Color32::from_gray(70)),
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

                                    if hitbox_editor.show_hitboxes {
                                        let image_dims = loaded_image_sizes
                                            .get(&normalized)
                                            .map(|(w, h)| egui::vec2(*w as f32, *h as f32))
                                            .unwrap_or_else(|| egui::vec2(display_size.x, display_size.y));
                                        let ratio_units_per_pixel = units_per_pixel(et_ref.height, image_dims.y);

                                        let mut rect = hitbox_editor
                                            .edited_hitboxes
                                            .get(&state_key)
                                            .copied()
                                            .unwrap_or_else(|| {
                                                RectHitbox::from_points(
                                                    &state_def.hitbox,
                                                    image_dims.x,
                                                    image_dims.y,
                                                    ratio_units_per_pixel,
                                                )
                                            });

                                        rect.clamp_to_image(image_dims.x, image_dims.y, ratio_units_per_pixel);
                                        let screen_hitbox = hitbox_to_screen_with_ratio(
                                            rect,
                                            image_rect,
                                            image_dims,
                                            ratio_units_per_pixel,
                                        );

                                        if let Some(pointer_pos) = response.hover_pos() {
                                            if let Some(edge) = pick_hitbox_edge(pointer_pos, screen_hitbox) {
                                                response = response.on_hover_cursor(cursor_for_drag_edge(edge));
                                            }
                                        }

                                        ui.painter().rect_stroke(
                                            screen_hitbox,
                                            0.0,
                                            egui::Stroke::new(2.0, egui::Color32::RED),
                                            egui::StrokeKind::Inside,
                                        );

                                        if response.drag_started() {
                                            if let Some(pointer_pos) = response.interact_pointer_pos() {
                                                if let Some(edge) = pick_hitbox_edge(pointer_pos, screen_hitbox) {
                                                    hitbox_editor.active_drag = Some(ActiveHitboxDrag {
                                                        state_key: state_key.clone(),
                                                        edge,
                                                    });
                                                }
                                            }
                                        }

                                        if response.dragged() {
                                            if let Some(active_drag) = &hitbox_editor.active_drag {
                                                if active_drag.state_key == state_key {
                                                    let pointer_delta = ui.ctx().input(|input| input.pointer.delta());
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
                                                        image_delta.x * ratio_units_per_pixel,
                                                        image_delta.y * ratio_units_per_pixel,
                                                    );

                                                    rect.drag_edge(
                                                        active_drag.edge,
                                                        units_delta,
                                                        image_dims.x,
                                                        image_dims.y,
                                                        ratio_units_per_pixel,
                                                    );
                                                    hitbox_editor.dirty_states.insert(state_key.clone());
                                                    // Mark the whole entity type as dirty so the title shows '*' and
                                                    // Ctrl+S will save staged changes.
                                                    hitbox_editor.dirty_entity_types.insert(selected_name.clone());
                                                }
                                            }
                                        }

                                        hitbox_editor.edited_hitboxes.insert(state_key.clone(), rect);
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
                        // Perform save logic (same as Ctrl+S handler)
                        // Prepare components to save: prefer staged copy, then document copy
                        let components_to_save: Option<Vec<String>> = if let Some(doc_ref) = document.as_ref() {
                            doc_ref.entity_types.get(&selected_name).map(|et| et.components.clone())
                        } else {
                            hitbox_editor
                                .edited_entity_types
                                .get(&selected_name)
                                .map(|et| et.components.clone())
                        };

                        let comp_result = if let Some(components) = components_to_save {
                            crate::io::save_entity_type_components(&selected_name, &components)
                        } else {
                            Ok(())
                        };

                        // Prepare hitbox save map
                        let mut save_map: HashMap<String, [[f32; 2]; 4]> = HashMap::new();
                        for state_key in &hitbox_editor.dirty_states {
                            if let Some(rect) = hitbox_editor.edited_hitboxes.get(state_key) {
                                save_map.insert(state_key.clone(), rect.to_json_points());
                            }
                        }

                        let hitbox_result = if save_map.is_empty() {
                            Ok(())
                        } else {
                            crate::io::save_entity_type_hitboxes(&selected_name, &save_map)
                        };

                        match comp_result.and(hitbox_result) {
                            Ok(()) => {
                                hitbox_editor.dirty_states.clear();
                                hitbox_editor.dirty_entity_types.remove(&selected_name);
                                *show_close_confirm = false;
                                next_state.set(crate::editor::EditorMode::LevelPicker);
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
                        hitbox_editor.edited_entity_types.remove(&selected_name);
                        hitbox_editor.dirty_entity_types.remove(&selected_name);
                        hitbox_editor.dirty_states.clear();
                        *show_close_confirm = false;
                        next_state.set(crate::editor::EditorMode::LevelPicker);
                    }

                    if ui.button("Cancel").clicked() {
                        *show_close_confirm = false;
                    }
                });
            });
    }
}



