use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
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

#[derive(Default)]
pub(crate) struct HitboxEditorState {
    show_hitboxes: bool,
    active_drag: Option<ActiveHitboxDrag>,
    edited_hitboxes: HashMap<String, RectHitbox>,
    dirty_states: HashSet<String>,
    last_entity_type: Option<String>,
    status_message: Option<String>,
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
    document: Option<Res<crate::editor::EditorDocument>>,
    mut next_state: ResMut<NextState<crate::editor::EditorMode>>,
    mut loaded_textures: Local<HashMap<String, TextureId>>,
    mut loaded_image_sizes: Local<HashMap<String, (u32, u32)>>,
    mut hitbox_editor: Local<HitboxEditorState>,
    asset_server: Res<AssetServer>,
) {
    // If nothing is selected, simply show a small message and return.
    if view_state.selected.is_none() {
        let ctx = contexts.ctx_mut();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Kein Entity-Type ausgewählt.");
        });
        return;
    }

    let selected_name = view_state.selected.clone().unwrap();

    if hitbox_editor.last_entity_type.as_deref() != Some(selected_name.as_str()) {
        hitbox_editor.edited_hitboxes.clear();
        hitbox_editor.dirty_states.clear();
        hitbox_editor.active_drag = None;
        hitbox_editor.status_message = None;
        hitbox_editor.last_entity_type = Some(selected_name.clone());
    }

    // Try to obtain the EntityTypeDefinition from the loaded EditorDocument
    // if present. Otherwise attempt to read the JSON file directly from
    // assets/entity_types/<selected>.json so the dashboard click works without
    // opening a level.
    let et_data: Cow<'_, crate::model::EntityTypeDefinition> = if let Some(doc) = &document {
        if let Some(et) = doc.entity_types.get(&selected_name) {
            Cow::Borrowed(et)
        } else {
            let ctx = contexts.ctx_mut();
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Entity-Type nicht in der geladenen Dokumentenliste gefunden.");
            });
            return;
        }
    } else {
        // No document loaded: try to read from assets/entity_types/<selected>.json
        let assets_dir = crate::io::assets_dir();
        let json_path = assets_dir.join("entity_types").join(format!("{}.json", selected_name));
        if !json_path.exists() {
            let ctx = contexts.ctx_mut();
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!("Entity-Type JSON nicht gefunden: {}", json_path.display()));
            });
            return;
        }

        let content = match std::fs::read_to_string(&json_path) {
            Ok(c) => c,
            Err(e) => {
                let ctx = contexts.ctx_mut();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!("Fehler beim Lesen von {}: {}", json_path.display(), e));
                });
                return;
            }
        };

        let parsed: Result<crate::model::EntityTypeDefinition, _> = serde_json::from_str(&content);
        let parsed = match parsed {
            Ok(p) => p,
            Err(e) => {
                let ctx = contexts.ctx_mut();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!("Fehler beim Parsen von {}: {}", json_path.display(), e));
                });
                return;
            }
        };

        // Basic validation similar to io::validate_entity_type_definition
        if parsed.states.is_empty() || !parsed.states.contains_key("default") {
            let ctx = contexts.ctx_mut();
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!(
                    "Entity-Type '{}' erfordert ein non-empty 'states' Objekt mit 'default'",
                    selected_name
                ));
            });
            return;
        }

        Cow::Owned(parsed)
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
            let tex_id = contexts.add_image(handle);
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

    let ctx = contexts.ctx_mut();

    if !ctx.input(|input| input.pointer.primary_down()) {
        hitbox_editor.active_drag = None;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::L)) {
        hitbox_editor.show_hitboxes = !hitbox_editor.show_hitboxes;
    }

    if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
        if hitbox_editor.dirty_states.is_empty() {
            hitbox_editor.status_message = Some("Keine Hitbox-Änderungen zum Speichern.".to_string());
        } else {
            let mut save_map: HashMap<String, [[f32; 2]; 4]> = HashMap::new();
            for state_key in &hitbox_editor.dirty_states {
                if let Some(rect) = hitbox_editor.edited_hitboxes.get(state_key) {
                    save_map.insert(state_key.clone(), rect.to_json_points());
                }
            }

            match crate::io::save_entity_type_hitboxes(&selected_name, &save_map) {
                Ok(()) => {
                    hitbox_editor.dirty_states.clear();
                    hitbox_editor.status_message = Some("Hitbox gespeichert.".to_string());
                }
                Err(error) => {
                    hitbox_editor.status_message = Some(format!("Speichern fehlgeschlagen: {error}"));
                }
            }
        }
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        // Make the entire central content scrollable
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Zurück zum Dashboard").clicked() {
                    next_state.set(crate::editor::EditorMode::LevelPicker);
                }
                ui.add_space(6.0);
                ui.heading("Entity-Type Vorschau");
            });
            ui.add_space(6.0);

            ui.label("L: Hitbox anzeigen/verstecken | Kante ziehen: Hitbox anpassen | Ctrl+S: speichern");
            if let Some(message) = &hitbox_editor.status_message {
                ui.label(message);
            }

            ui.label(format!("Ausgewählt: {}", selected_name));
            ui.separator();

            // Components
            ui.group(|ui| {
                ui.label(egui::RichText::new("Components").strong());
                ui.horizontal_wrapped(|ui| {
                    for comp in &et_ref.components {
                        ui.label(format!("[{}]", comp));
                    }
                });
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
                            ui.label("(keine Animation / keine Bilder)");
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
                                        ui.label("(Vorschau nicht geladen)");
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
}



