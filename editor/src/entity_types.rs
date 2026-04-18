use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};
use egui::TextureId;
use serde_json::Value;
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
    remove_component_confirm: Option<String>,
    json_editor_state: HashMap<String, String>,
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
            remove_component_confirm: None,
            json_editor_state: HashMap::new(),
            dirty_entity_types: HashSet::new(),
            edited_entity_types: HashMap::new(),
        }
    }
}

fn cloned_staged_entity_type(
    document: Option<&crate::editor::EditorDocument>,
    hitbox_editor: &HitboxEditorState,
    selected_name: &str,
    fallback: &crate::model::EntityTypeDefinition,
) -> crate::model::EntityTypeDefinition {
    if let Some(doc) = document {
        return doc
            .entity_types
            .get(selected_name)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
    }

    hitbox_editor
        .edited_entity_types
        .get(selected_name)
        .cloned()
        .unwrap_or_else(|| fallback.clone())
}

fn apply_to_staged_entity_type(
    document: Option<&mut crate::editor::EditorDocument>,
    hitbox_editor: &mut HitboxEditorState,
    selected_name: &str,
    fallback: &crate::model::EntityTypeDefinition,
    mutator: impl FnOnce(&mut crate::model::EntityTypeDefinition),
) -> bool {
    if let Some(doc) = document {
        if let Some(et) = doc.entity_types.get_mut(selected_name) {
            mutator(et);
            return true;
        }
        return false;
    }

    let et = hitbox_editor
        .edited_entity_types
        .entry(selected_name.to_string())
        .or_insert_with(|| fallback.clone());
    mutator(et);
    true
}

fn component_object_snapshot(
    entity_type: &crate::model::EntityTypeDefinition,
    component_name: &str,
) -> Option<serde_json::Map<String, Value>> {
    let components_obj = entity_type
        .components
        .as_ref()
        .and_then(|components| serde_json::to_value(components).ok())
        .and_then(|value| value.as_object().cloned())?;

    components_obj
        .get(component_name)
        .and_then(|value| value.as_object().cloned())
}

fn component_default_value(
    component_name: &str,
    attribute_name: &str,
) -> Option<Value> {
    let mut probe = crate::model::EntityTypeDefinition {
        components: None,
        category_tag: None,
        width: None,
        height: None,
        key: String::new(),
    };
    probe.set_component_names(&[component_name.to_string()]);
    probe
        .component_attribute_value(component_name, attribute_name)
        .and_then(|value| if value.is_null() { None } else { Some(value) })
}

fn save_staged_entity_type(
    document: Option<&crate::editor::EditorDocument>,
    hitbox_editor: &HitboxEditorState,
    selected_name: &str,
    fallback: &crate::model::EntityTypeDefinition,
) -> Result<(), String> {
    let mut to_save = cloned_staged_entity_type(document, hitbox_editor, selected_name, fallback);

    if !hitbox_editor.dirty_states.is_empty() {
        let mut state_machine = to_save
            .state_machine()
            .ok_or_else(|| "Cannot save hitboxes: missing state_machine component".to_string())?;

        for state_key in &hitbox_editor.dirty_states {
            if let Some(rect) = hitbox_editor.edited_hitboxes.get(state_key) {
                let state = state_machine
                    .states
                    .get_mut(state_key)
                    .ok_or_else(|| format!("Cannot save hitbox: missing state '{state_key}'"))?;
                state.collider_box = Some(rect.to_json_points().to_vec());
            }
        }

        to_save
            .set_state_machine(state_machine)
            .map_err(|error| error.to_string())?;
    }

    crate::io::save_entity_type_definition(selected_name, &to_save)
}

#[derive(Clone)]
struct AttributeUiRow {
    name: String,
    attr_type: String,
    options: Vec<String>,
}

fn sorted_attribute_rows(
    mapping: &crate::editor::ComponentValueMapping,
    entity_type: &crate::model::EntityTypeDefinition,
    component_name: &str,
) -> Vec<AttributeUiRow> {
    let mut rows: Vec<AttributeUiRow> = Vec::new();
    let mut seen = HashSet::<String>::new();

    if let Some(component_mapping) = mapping.components.get(component_name) {
        let mut mapped_rows: Vec<AttributeUiRow> = component_mapping
            .iter()
            .map(|(name, def)| AttributeUiRow {
                name: name.clone(),
                attr_type: def.attr_type.clone(),
                options: def.options.clone(),
            })
            .collect();
        mapped_rows.sort_by(|left, right| left.name.cmp(&right.name));

        for row in mapped_rows {
            seen.insert(row.name.clone());
            rows.push(row);
        }
    }

    if let Some(component_object) = component_object_snapshot(entity_type, component_name) {
        let mut fallback_keys: Vec<String> = component_object
            .keys()
            .filter(|key| !seen.contains(*key))
            .cloned()
            .collect();
        fallback_keys.sort();

        for key in fallback_keys {
            let inferred_type = component_object
                .get(&key)
                .map(|value| {
                    if key.eq_ignore_ascii_case("waypoints") {
                        return "waypoints".to_string();
                    }

                    if let Some(items) = value.as_array() {
                        let is_waypoints = items.iter().all(|item| {
                            item
                                .as_array()
                                .map(|pair| {
                                    pair.len() == 2
                                        && pair.first().and_then(|v| v.as_f64()).is_some()
                                        && pair.get(1).and_then(|v| v.as_f64()).is_some()
                                })
                                .unwrap_or(false)
                        });
                        if is_waypoints {
                            return "waypoints".to_string();
                        }

                        let is_number_array = items.iter().all(|item| item.as_f64().is_some());
                        if is_number_array {
                            return "array<number>".to_string();
                        }

                        let is_string_array = items.iter().all(|item| item.as_str().is_some());
                        if is_string_array {
                            return "array<string>".to_string();
                        }

                        return "array".to_string();
                    }

                    if value.as_f64().is_some() {
                        "number".to_string()
                    } else if value.as_str().is_some() {
                        "string".to_string()
                    } else {
                        "json".to_string()
                    }
                })
                .unwrap_or_else(|| "json".to_string());

            rows.push(AttributeUiRow {
                name: key,
                attr_type: inferred_type,
                options: Vec::new(),
            });
        }
    }

    rows
}

fn render_components_sidebar(
    ctx: &egui::Context,
    selected_name: &str,
    document: &mut Option<ResMut<crate::editor::EditorDocument>>,
    hitbox_editor: &mut HitboxEditorState,
    fallback_entity_type: &crate::model::EntityTypeDefinition,
    mapping: &crate::editor::ComponentValueMapping,
    toast: &mut crate::editor::ToastState,
    time: &Time,
) {
    egui::SidePanel::right("entity_type_components_sidebar")
        .resizable(true)
        .default_width(300.0)
        .min_width(200.0)
        .max_width(600.0)
        .show(ctx, |ui| {
            ui.heading("Components");
            ui.add_space(6.0);

            let available_components = match crate::io::scan_game_components() {
                Ok(v) => v,
                Err(e) => {
                    toast.message = Some(format!("Could not scan components: {}", e));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    Vec::new()
                }
            };

            let staged_snapshot = cloned_staged_entity_type(
                document.as_deref(),
                hitbox_editor,
                selected_name,
                fallback_entity_type,
            );
            let components_snapshot = staged_snapshot.component_names();

            let add_options: Vec<String> = available_components
                .into_iter()
                .filter(|name| !components_snapshot.iter().any(|existing| existing == name))
                .collect();

            ui.horizontal(|ui| {
                ui.label("Add component:");
                let mut selected = hitbox_editor.add_selected.clone().unwrap_or_default();
                egui::ComboBox::from_id_salt(format!("add_component_cb_{}", selected_name))
                    .selected_text(if selected.is_empty() { "select..." } else { &selected })
                    .show_ui(ui, |ui| {
                        for option in &add_options {
                            ui.selectable_value(&mut selected, option.clone(), option);
                        }
                    });
                hitbox_editor.add_selected = if selected.is_empty() { None } else { Some(selected.clone()) };

                // Show the Add button immediately to the right of the ComboBox.
                let add_enabled = hitbox_editor
                    .add_selected
                    .as_ref()
                    .map(|selection| add_options.iter().any(|option| option == selection))
                    .unwrap_or(false);

                if ui.add_enabled(add_enabled, egui::Button::new("Add")).clicked() {
                    if let Some(chosen) = hitbox_editor.add_selected.clone() {
                        let mut new_components = components_snapshot.clone();
                        if !new_components.iter().any(|component| component == &chosen) {
                            new_components.push(chosen.clone());
                            if apply_to_staged_entity_type(
                                document.as_deref_mut(),
                                hitbox_editor,
                                selected_name,
                                fallback_entity_type,
                                |et| et.set_component_names(&new_components),
                            ) {
                                hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                            }
                        }
                    }
                    hitbox_editor.add_selected = None;
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for component_name in &components_snapshot {
                    let attr_rows = sorted_attribute_rows(mapping, &staged_snapshot, component_name);

                    egui::CollapsingHeader::new(component_name)
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(2.0);
                                if ui.small_button("Remove").clicked() {
                                    hitbox_editor.remove_component_confirm = Some(component_name.clone());
                                }
                            });

                            for row in &attr_rows {
                                let explicit_value = staged_snapshot
                                    .component_attribute_value(component_name, &row.name);
                                let component_default = component_default_value(component_name, &row.name);
                                let enum_default = if row.attr_type == "enum" {
                                    row.options.first().cloned().map(Value::String)
                                } else {
                                    None
                                };
                                let display_default = component_default.clone().or(enum_default.clone());

                                ui.horizontal(|ui| {
                                    ui.add_sized([150.0, 20.0], egui::Label::new(&row.name));

                                    match row.attr_type.as_str() {
                                        "number" => {
                                            let mut value = explicit_value
                                                .as_ref()
                                                .and_then(|v| v.as_f64())
                                                .or_else(|| display_default.as_ref().and_then(|v| v.as_f64()))
                                                .unwrap_or(0.0);
                                            if ui.add(egui::DragValue::new(&mut value).speed(0.1)).changed() {
                                                if let Some(num) = serde_json::Number::from_f64(value) {
                                                    if apply_to_staged_entity_type(
                                                        document.as_deref_mut(),
                                                        hitbox_editor,
                                                        selected_name,
                                                        fallback_entity_type,
                                                        |et| {
                                                            et.set_component_attribute_value(
                                                                component_name,
                                                                &row.name,
                                                                Value::Number(num),
                                                            )
                                                        },
                                                    ) {
                                                        hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                        "string" => {
                                            let mut text = explicit_value
                                                .as_ref()
                                                .and_then(|v| v.as_str())
                                                .or_else(|| display_default.as_ref().and_then(|v| v.as_str()))
                                                .unwrap_or("")
                                                .to_string();
                                            if ui.text_edit_singleline(&mut text).changed() {
                                                if apply_to_staged_entity_type(
                                                    document.as_deref_mut(),
                                                    hitbox_editor,
                                                    selected_name,
                                                    fallback_entity_type,
                                                    |et| {
                                                        et.set_component_attribute_value(
                                                            component_name,
                                                            &row.name,
                                                            Value::String(text),
                                                        )
                                                    },
                                                ) {
                                                    hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                }
                                            }
                                        }
                                        "enum" => {
                                            let mut current = explicit_value
                                                .as_ref()
                                                .and_then(|v| v.as_str())
                                                .map(|v| v.to_string())
                                                .or_else(|| {
                                                    display_default
                                                        .as_ref()
                                                        .and_then(|v| v.as_str())
                                                        .map(|v| v.to_string())
                                                })
                                                .or_else(|| row.options.first().cloned())
                                                .unwrap_or_default();
                                            let before_current = current.clone();

                                            egui::ComboBox::from_id_salt(format!(
                                                "entity_type_enum_{}_{}_{}",
                                                selected_name, component_name, row.name
                                            ))
                                            .selected_text(if current.is_empty() {
                                                "select..."
                                            } else {
                                                &current
                                            })
                                            .show_ui(ui, |ui| {
                                                for option in &row.options {
                                                    ui.selectable_value(&mut current, option.clone(), option);
                                                }
                                            });

                                            if !current.is_empty() && current != before_current
                                            {
                                                if apply_to_staged_entity_type(
                                                    document.as_deref_mut(),
                                                    hitbox_editor,
                                                    selected_name,
                                                    fallback_entity_type,
                                                    |et| {
                                                        et.set_component_attribute_value(
                                                            component_name,
                                                            &row.name,
                                                            Value::String(current),
                                                        )
                                                    },
                                                ) {
                                                    hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                }
                                            }
                                        }
                                        "waypoints" => {
                                            let mut points = explicit_value
                                                .as_ref()
                                                .and_then(|v| v.as_array().cloned())
                                                .or_else(|| display_default.as_ref().and_then(|v| v.as_array().cloned()))
                                                .unwrap_or_default();

                                            ui.vertical(|ui| {
                                                let mut changed = false;
                                                let mut remove_index: Option<usize> = None;
                                                let mut move_up: Option<usize> = None;
                                                let mut move_down: Option<usize> = None;
                                                let len = points.len();

                                                for (index, point_value) in points.iter_mut().enumerate() {
                                                    let mut x = point_value
                                                        .as_array()
                                                        .and_then(|pair| pair.first())
                                                        .and_then(|v| v.as_f64())
                                                        .unwrap_or(0.0);
                                                    let mut y = point_value
                                                        .as_array()
                                                        .and_then(|pair| pair.get(1))
                                                        .and_then(|v| v.as_f64())
                                                        .unwrap_or(0.0);

                                                    ui.horizontal(|ui| {
                                                        changed |= ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                                                        changed |= ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();
                                                        if ui.small_button("↑").clicked() && index > 0 {
                                                            move_up = Some(index);
                                                        }
                                                        if ui.small_button("↓").clicked() && index + 1 < len {
                                                            move_down = Some(index);
                                                        }
                                                        if ui.small_button("-").clicked() {
                                                            remove_index = Some(index);
                                                        }
                                                    });

                                                    if let (Some(nx), Some(ny)) = (
                                                        serde_json::Number::from_f64(x),
                                                        serde_json::Number::from_f64(y),
                                                    ) {
                                                        *point_value = Value::Array(vec![Value::Number(nx), Value::Number(ny)]);
                                                    }
                                                }

                                                if let Some(index) = remove_index {
                                                    points.remove(index);
                                                    changed = true;
                                                }
                                                if let Some(index) = move_up {
                                                    points.swap(index, index - 1);
                                                    changed = true;
                                                }
                                                if let Some(index) = move_down {
                                                    points.swap(index, index + 1);
                                                    changed = true;
                                                }

                                                if ui.small_button("Add waypoint").clicked() {
                                                    points.push(Value::Array(vec![
                                                        Value::Number(serde_json::Number::from(0)),
                                                        Value::Number(serde_json::Number::from(0)),
                                                    ]));
                                                    changed = true;
                                                }

                                                if changed {
                                                    if apply_to_staged_entity_type(
                                                        document.as_deref_mut(),
                                                        hitbox_editor,
                                                        selected_name,
                                                        fallback_entity_type,
                                                        |et| {
                                                            et.set_component_attribute_value(
                                                                component_name,
                                                                &row.name,
                                                                Value::Array(points),
                                                            )
                                                        },
                                                    ) {
                                                        hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                    }
                                                }
                                            });
                                        }
                                        attr if attr.starts_with("array") => {
                                            let mut values = explicit_value
                                                .as_ref()
                                                .and_then(|v| v.as_array().cloned())
                                                .or_else(|| display_default.as_ref().and_then(|v| v.as_array().cloned()))
                                                .unwrap_or_default();

                                            ui.vertical(|ui| {
                                                let is_number_array = attr.contains("number");
                                                let mut changed = false;
                                                let mut remove_index: Option<usize> = None;
                                                let mut move_up: Option<usize> = None;
                                                let mut move_down: Option<usize> = None;
                                                let len = values.len();

                                                for (index, value) in values.iter_mut().enumerate() {
                                                    ui.horizontal(|ui| {
                                                        if is_number_array {
                                                            let mut num = value.as_f64().unwrap_or(0.0);
                                                            if ui.add(egui::DragValue::new(&mut num).speed(0.1)).changed() {
                                                                if let Some(number) = serde_json::Number::from_f64(num) {
                                                                    *value = Value::Number(number);
                                                                    changed = true;
                                                                }
                                                            }
                                                        } else {
                                                            let mut text = value.as_str().unwrap_or("").to_string();
                                                            if ui.text_edit_singleline(&mut text).changed() {
                                                                *value = Value::String(text);
                                                                changed = true;
                                                            }
                                                        }

                                                        if ui.small_button("↑").clicked() && index > 0 {
                                                            move_up = Some(index);
                                                        }
                                                        if ui.small_button("↓").clicked() && index + 1 < len {
                                                            move_down = Some(index);
                                                        }
                                                        if ui.small_button("-").clicked() {
                                                            remove_index = Some(index);
                                                        }
                                                    });
                                                }

                                                if let Some(index) = remove_index {
                                                    values.remove(index);
                                                    changed = true;
                                                }
                                                if let Some(index) = move_up {
                                                    values.swap(index, index - 1);
                                                    changed = true;
                                                }
                                                if let Some(index) = move_down {
                                                    values.swap(index, index + 1);
                                                    changed = true;
                                                }

                                                if ui.small_button("Add").clicked() {
                                                    if is_number_array {
                                                        values.push(Value::Number(serde_json::Number::from(0)));
                                                    } else {
                                                        values.push(Value::String(String::new()));
                                                    }
                                                    changed = true;
                                                }

                                                if changed {
                                                    if apply_to_staged_entity_type(
                                                        document.as_deref_mut(),
                                                        hitbox_editor,
                                                        selected_name,
                                                        fallback_entity_type,
                                                        |et| {
                                                            et.set_component_attribute_value(
                                                                component_name,
                                                                &row.name,
                                                                Value::Array(values),
                                                            )
                                                        },
                                                    ) {
                                                        hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                    }
                                                }
                                            });
                                        }
                                        _ => {
                                            let editor_key = format!(
                                                "json::{}::{}::{}",
                                                selected_name, component_name, row.name
                                            );
                                            let initial_text = explicit_value
                                                .as_ref()
                                                .or(display_default.as_ref())
                                                .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string()))
                                                .unwrap_or_else(|| "null".to_string());
                                            let mut text_value = hitbox_editor
                                                .json_editor_state
                                                .get(&editor_key)
                                                .cloned()
                                                .unwrap_or(initial_text);

                                            let response = ui.add(
                                                egui::TextEdit::multiline(&mut text_value)
                                                    .desired_width(220.0)
                                                    .desired_rows(3),
                                            );
                                            if response.changed() {
                                                hitbox_editor
                                                    .json_editor_state
                                                    .insert(editor_key, text_value.clone());

                                                if let Ok(parsed) = serde_json::from_str::<Value>(&text_value) {
                                                    if apply_to_staged_entity_type(
                                                        document.as_deref_mut(),
                                                        hitbox_editor,
                                                        selected_name,
                                                        fallback_entity_type,
                                                        |et| {
                                                            et.set_component_attribute_value(
                                                                component_name,
                                                                &row.name,
                                                                parsed,
                                                            )
                                                        },
                                                    ) {
                                                        hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if explicit_value.is_some() {
                                        if ui.small_button("Clear").on_hover_text(
                                            "Reset to default (removes explicit override from JSON)",
                                        ).clicked() {
                                            if apply_to_staged_entity_type(
                                                document.as_deref_mut(),
                                                hitbox_editor,
                                                selected_name,
                                                fallback_entity_type,
                                                |et| et.remove_component_attribute(component_name, &row.name),
                                            ) {
                                                hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                            }
                                        }
                                    }
                                });

                                if explicit_value.is_none() {
                                    let hint = if component_default.is_some() {
                                        Some("component default")
                                    } else if enum_default.is_some() {
                                        Some("first enum option")
                                    } else {
                                        None
                                    };

                                    ui.horizontal(|ui| {
                                        ui.add_space(152.0);
                                        if let Some(source) = hint {
                                            ui.label(
                                                egui::RichText::new("default")
                                                    .weak()
                                                    .italics(),
                                            )
                                            .on_hover_text(source);
                                        } else {
                                            ui.label(
                                                egui::RichText::new("(not set)")
                                                    .weak()
                                                    .italics(),
                                            );
                                        }
                                    });
                                }
                            }
                        });

                    ui.separator();
                }
            });
        });
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
    mapping: Res<crate::editor::ComponentValueMapping>,
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
        if state_machine.states.is_empty() || !state_machine.states.contains_key(&state_machine.initial_state) {
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
        hitbox_editor
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
            match save_staged_entity_type(
                document.as_ref().map(|doc| doc.as_ref()),
                &hitbox_editor,
                &selected_name,
                et_ref,
            ) {
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

    render_components_sidebar(
        ctx,
        &selected_name,
        &mut document,
        &mut hitbox_editor,
        et_ref,
        &mapping,
        &mut toast,
        &time,
    );

    egui::CentralPanel::default().show(ctx, |ui| {
        // Make the entire central content scrollable
        egui::ScrollArea::vertical().show(ui, |ui| {
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
                                        let ratio_units_per_pixel =
                                            units_per_pixel(et_ref.height.unwrap_or_default(), image_dims.y);

                                        let mut rect = hitbox_editor
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

    if let Some(component_name) = hitbox_editor.remove_component_confirm.clone() {
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
                            &hitbox_editor,
                            &selected_name,
                            et_ref,
                        );
                        let mut new_components = snapshot.component_names();
                        new_components.retain(|name| name != &component_name);

                        if apply_to_staged_entity_type(
                            document.as_deref_mut(),
                            &mut hitbox_editor,
                            &selected_name,
                            et_ref,
                            |et| et.set_component_names(&new_components),
                        ) {
                            hitbox_editor.dirty_entity_types.insert(selected_name.clone());
                        }
                        hitbox_editor.remove_component_confirm = None;
                    }

                    if ui.button("Cancel").clicked() {
                        hitbox_editor.remove_component_confirm = None;
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
                            &hitbox_editor,
                            &selected_name,
                            et_ref,
                        ) {
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



