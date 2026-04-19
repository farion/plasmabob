use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};
use egui_extras::{TableBuilder, Column};
use egui::TextureId;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

const HITBOX_EDGE_PICK_TOLERANCE_PX: f32 = 12.0;
const HITBOX_MIN_SIZE_PX: f32 = 1.0;
const PREVIEW_CANVAS_WIDTH_PX: f32 = 512.0;
const PREVIEW_CANVAS_HEIGHT_PX: f32 = 256.0;

    // Stable layout widths for the attribute table columns. These must remain
    // constant across all component categories so columns don't shift when
    // switching between components. Values are provided by the shared
    // ColumnWidths resource so other UIs can stay consistent.

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
    collapsed_components: HashSet<String>,
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
            collapsed_components: HashSet::new(),
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
    fallback_entity: &crate::model::EntityTypeDefinition,
    component_name: &str,
) -> Vec<AttributeUiRow> {
    let mut rows: Vec<AttributeUiRow> = Vec::new();
    let mut seen = HashSet::<String>::new();

    if let Some(component_mapping) = mapping.components.get(component_name) {
        let mut mapped_rows: Vec<AttributeUiRow> = component_mapping
            .iter()
            // Only include mapping entries that actually exist on the
            // component config structs. This prevents stale or incorrect
            // entries in component_value_mapping.json from exposing
            // attributes that don't belong to the typed configs.
            .filter_map(|(name, def)| {
                // Allow mapping entries for components that intentionally
                // accept arbitrary keys (e.g. collider uses a flattened map).
                let allow_extra = matches!(component_name.to_ascii_lowercase().as_str(), "collider");
                if allow_extra || component_attribute_type(component_name, name).is_some() {
                    Some(AttributeUiRow {
                        name: name.clone(),
                        attr_type: def.attr_type.clone(),
                        options: def.options.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();
        mapped_rows.sort_by(|left, right| left.name.cmp(&right.name));

        for row in mapped_rows {
            seen.insert(row.name.clone());
            rows.push(row);
        }
    }

    // Collect keys from the original/fallback entity only (not staged edits).
    // This prevents the attribute's UI type from changing when a staged
    // explicit value is cleared in the editor.
    let mut combined_keys = std::collections::HashSet::<String>::new();
    if let Some(fallback_obj) = component_object_snapshot(fallback_entity, component_name) {
        for k in fallback_obj.keys() {
            combined_keys.insert(k.clone());
        }
    }

    let mut fallback_keys: Vec<String> = combined_keys
        .into_iter()
        .filter(|key| !seen.contains(key))
        .collect();
    fallback_keys.sort();

    for key in fallback_keys {
        // Determine attribute type from component config structs (source of
        // truth). If the attribute does not exist on the config struct we
        // skip it entirely. This guarantees the editor only exposes fields
        // actually defined on the typed ComponentConfig structs.
        if let Some(attr_type) = component_attribute_type(component_name, &key) {
            seen.insert(key.clone());
            rows.push(AttributeUiRow {
                name: key,
                attr_type: attr_type.to_string(),
                options: Vec::new(),
            });
        }
    }

    // Ensure attributes declared in the typed ComponentConfig structs are
    // always present even when the serialized fallback object is empty
    // (fields are Option<T> and therefore serialize to an empty object).
    for &(name, typ) in component_declared_attributes(component_name) {
        if !seen.contains(&name.to_string()) {
            seen.insert(name.to_string());
            rows.push(AttributeUiRow { name: name.to_string(), attr_type: typ.to_string(), options: Vec::new() });
        }
    }

    // Final defensive dedup: preserve first occurrence when duplicates
    // somehow slipped through earlier merging logic (mapping + fallback + declared).
    let mut out: Vec<AttributeUiRow> = Vec::new();
    let mut out_seen = HashSet::<String>::new();
    for row in rows.into_iter() {
        if out_seen.insert(row.name.clone()) {
            out.push(row);
        }
    }

    out
}

include!(concat!(env!("OUT_DIR"), "/component_attr_map.rs"));

// Return attributes declared by the component config structs so the editor
// can include fields that serialize as absent when they are `Option<T>`
// and therefore don't appear in a serialized fallback object.
// component_declared_attributes is provided by build.rs generated file

fn render_components_sidebar(
    ctx: &egui::Context,
    selected_name: &str,
    document: &mut Option<ResMut<crate::editor::EditorDocument>>,
    hitbox_editor: &mut HitboxEditorState,
    fallback_entity_type: &crate::model::EntityTypeDefinition,
    mapping: &crate::editor::ComponentValueMapping,
    toast: &mut crate::editor::ToastState,
    time: &Time,
    mut widths: ResMut<crate::editor::table_ui::ColumnWidths>,
) {
    egui::SidePanel::right("entity_type_components_sidebar")
        .resizable(true)
        // Allow the sidebar to be between 300 and 600 px wide. Default to 300.
        .default_width(300.0)
        .min_width(300.0)
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

                if ui.add_enabled(add_enabled, egui::Button::new(egui_phosphor_icons::icons::PLUS)).clicked() {
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

            // Capture the panel's available width *before* the ScrollArea so that
            // all column widths are computed from the current panel size. Using
            // explicit widths (Column::exact) instead of Column::remainder breaks
            // the feedback loop that otherwise prevents the sidebar from shrinking.
            let button_padding = 8.0f32;
            let button_w = 24.0f32;
            let clear_col_w = button_padding * 2.0 + button_w;
            let name_col_w_global = widths.widths.get(0).cloned().unwrap_or(80.0);
            // Two inter-column gaps at egui's default item_spacing.x (8 px each) plus
            // a small margin so the table does not touch the panel edge.
            let col_spacing = ui.spacing().item_spacing.x * 2.0 + 4.0;
            let text_col_w_global = (ui.available_width() - name_col_w_global - clear_col_w - col_spacing).max(40.0);
            widths.widths = vec![name_col_w_global, text_col_w_global, clear_col_w];

            egui::ScrollArea::vertical()
                .id_salt(format!("entity_type_components_scroll_{}", selected_name))
                .show(ui, |ui| {

                // Revert to per-component header rendered manually so we can
                // place the remove (trash) button inline to the right of the
                // component name (CollapsingHeader places content under the
                // header which pushed the button below the label).
                for component_name in &components_snapshot {
                    let attr_rows = sorted_attribute_rows(mapping, &staged_snapshot, fallback_entity_type, component_name);

                    let component_scope_id = format!("entity_type_component_section_{}_{}", selected_name, component_name);
                    ui.push_id(component_scope_id, |ui| {

                    // Header: allocate a full-width rect and split it into a
                    // left area (arrow + name) and a right area (trash button).
                    let header_h = 24.0f32;
                    let header_size = egui::vec2(ui.available_width(), header_h);
                    let (header_rect, _header_resp) = ui.allocate_exact_size(header_size, egui::Sense::click());

                    let is_collapsed = hitbox_editor.collapsed_components.contains(component_name);

                    // Left area: leave room for the clear column on the right
                    let left_rect = egui::Rect::from_min_max(header_rect.min, egui::pos2(header_rect.max.x - clear_col_w, header_rect.max.y));
                    // Arrow on the left, label to the right. Use `ui.put` to place
                    // buttons exactly so icons are visually centered.
                    let arrow_icon = if is_collapsed { egui_phosphor_icons::icons::CARET_RIGHT } else { egui_phosphor_icons::icons::CARET_DOWN };
                    let arrow_rect = egui::Rect::from_min_max(
                        egui::pos2(left_rect.min.x + 4.0, left_rect.min.y),
                        egui::pos2(left_rect.min.x + 4.0 + button_w, left_rect.max.y),
                    );
                    let arrow_resp = ui.put(arrow_rect, egui::Button::new(arrow_icon).min_size(egui::vec2(button_w, header_h)));
                    if arrow_resp.clicked() {
                        if is_collapsed { hitbox_editor.collapsed_components.remove(component_name); } else { hitbox_editor.collapsed_components.insert(component_name.clone()); }
                    }

                    let label_rect = egui::Rect::from_min_max(
                        egui::pos2(arrow_rect.max.x + 6.0, left_rect.min.y),
                        egui::pos2(left_rect.max.x, left_rect.max.y),
                    );
                    ui.allocate_ui_at_rect(label_rect, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(component_name).strong());
                        });
                    });

                    // Right area: trash button placed centered in the clear column
                    let right_rect = egui::Rect::from_min_max(
                        egui::pos2(header_rect.max.x - clear_col_w, header_rect.min.y),
                        header_rect.max,
                    );
                    // center the button horizontally within the clear column for visual alignment
                    let btn_center_x = (right_rect.min.x + right_rect.max.x) * 0.5;
                    let button_rect = egui::Rect::from_min_max(
                        egui::pos2(btn_center_x - button_w * 0.5, right_rect.min.y),
                        egui::pos2(btn_center_x + button_w * 0.5, right_rect.max.y),
                    );
                    let trash_resp = ui.put(button_rect, egui::Button::new(egui_phosphor_icons::icons::TRASH).min_size(egui::vec2(button_w, header_h)));
                    if trash_resp.clicked() {
                        hitbox_editor.remove_component_confirm = Some(component_name.clone());
                    }

                    // Only render attributes when expanded.
                    if !hitbox_editor.collapsed_components.contains(component_name) {
                        let name_col_w = name_col_w_global;
                        let middle_col_w = text_col_w_global;
                        widths.widths = vec![name_col_w, middle_col_w, clear_col_w];

                        // Use exact column widths so no column can grow beyond the
                        // pre-computed values and force the sidebar wider.
                        let table = TableBuilder::new(ui).striped(true)
                            .column(Column::exact(name_col_w))
                            .column(Column::exact(middle_col_w))
                            .column(Column::exact(clear_col_w));

                        table.body(|mut body| {
                        for row in &attr_rows {
                                    let mut explicit_value = staged_snapshot
                                        .component_attribute_value(component_name, &row.name);
                                    let component_default = component_default_value(component_name, &row.name);
                                    let enum_default = if row.attr_type == "enum" {
                                        row.options.first().cloned().map(Value::String)
                                    } else {
                                        None
                                    };
                                    let display_default = component_default.clone().or(enum_default.clone());

                                    body.row(20.0, |mut r| {
                                        // Column 1: name
                                        r.col(|ui| {
                                            ui.label(&row.name);
                                        });

                                        // Column 2: widget (muted when not explicit)
                                        r.col(|ui| {
                                            let is_explicit = explicit_value.is_some();
                                            let saved_override = ui.visuals().override_text_color;
                                            if !is_explicit { ui.visuals_mut().override_text_color = Some(egui::Color32::from_gray(140)); }

                                            match row.attr_type.as_str() {
                                                "number" | "int" => {
                                                    let is_int = row.attr_type == "int";
                                                    let mut value_f = explicit_value
                                                        .as_ref()
                                                        .and_then(|v| v.as_f64())
                                                        .or_else(|| display_default.as_ref().and_then(|v| v.as_f64()))
                                                        .unwrap_or(0.0);
                                                    if ui.add(egui::DragValue::new(&mut value_f).speed(1.0)).changed() {
                                                        if is_int {
                                                            let int_val = value_f.round() as i64;
                                                            let num = serde_json::Number::from(int_val);
                                                            if apply_to_staged_entity_type(
                                                                document.as_deref_mut(),
                                                                hitbox_editor,
                                                                selected_name,
                                                                fallback_entity_type,
                                                                |et| { et.set_component_attribute_value(component_name, &row.name, Value::Number(num)) },
                                                            ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                        } else if let Some(numf) = serde_json::Number::from_f64(value_f) {
                                                            if apply_to_staged_entity_type(
                                                                document.as_deref_mut(),
                                                                hitbox_editor,
                                                                selected_name,
                                                                fallback_entity_type,
                                                                |et| { et.set_component_attribute_value(component_name, &row.name, Value::Number(numf)) },
                                                            ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
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
                                                     // Use the pre-computed column width so the field fills the column
                                                     // without requesting more space than the panel currently provides.
                                                     if ui.add(egui::TextEdit::singleline(&mut text).desired_width(middle_col_w)).changed() {
                                                        if apply_to_staged_entity_type(
                                                            document.as_deref_mut(),
                                                            hitbox_editor,
                                                            selected_name,
                                                            fallback_entity_type,
                                                            |et| { et.set_component_attribute_value(component_name, &row.name, Value::String(text)) },
                                                        ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                    }
                                                }
                                                "bool" => {
                                                    let mut checked = explicit_value
                                                        .as_ref()
                                                        .and_then(|v| v.as_bool())
                                                        .or_else(|| display_default.as_ref().and_then(|v| v.as_bool()))
                                                        .unwrap_or(false);
                                                    let before = checked;
                                                    if ui.add(egui::Checkbox::new(&mut checked, "")).changed() && checked != before {
                                                        if apply_to_staged_entity_type(
                                                            document.as_deref_mut(),
                                                            hitbox_editor,
                                                            selected_name,
                                                            fallback_entity_type,
                                                            |et| { et.set_component_attribute_value(component_name, &row.name, Value::Bool(checked)) },
                                                        ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                    }
                                                }
                                                "enum" => {
                                                    let mut current = explicit_value
                                                        .as_ref()
                                                        .and_then(|v| v.as_str())
                                                        .map(|v| v.to_string())
                                                        .or_else(|| display_default.as_ref().and_then(|v| v.as_str()).map(|v| v.to_string()))
                                                        .or_else(|| row.options.first().cloned())
                                                        .unwrap_or_default();
                                                    let before_current = current.clone();
                                                    egui::ComboBox::from_id_salt(format!("entity_type_enum_{}_{}_{}", selected_name, component_name, row.name))
                                                        .selected_text(if current.is_empty() { "select..." } else { &current })
                                                        .show_ui(ui, |ui| { for option in &row.options { ui.selectable_value(&mut current, option.clone(), option); } });
                                                    if !current.is_empty() && current != before_current {
                                                        if apply_to_staged_entity_type(
                                                            document.as_deref_mut(),
                                                            hitbox_editor,
                                                            selected_name,
                                                            fallback_entity_type,
                                                            |et| { et.set_component_attribute_value(component_name, &row.name, Value::String(current.clone())) },
                                                        ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
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
                                                            let mut x = point_value.as_array().and_then(|pair| pair.first()).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                            let mut y = point_value.as_array().and_then(|pair| pair.get(1)).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                            ui.horizontal(|ui| {
                                                                changed |= ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                                                                changed |= ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();
                                                                if ui.small_button("↑").clicked() && index > 0 { move_up = Some(index); }
                                                                if ui.small_button("↓").clicked() && index + 1 < len { move_down = Some(index); }
                                                                if ui.small_button("-").clicked() { remove_index = Some(index); }
                                                            });
                                                            if let (Some(nx), Some(ny)) = (serde_json::Number::from_f64(x), serde_json::Number::from_f64(y)) { *point_value = Value::Array(vec![Value::Number(nx), Value::Number(ny)]); }
                                                        }
                                                        if let Some(index) = remove_index { points.remove(index); changed = true; }
                                                        if let Some(index) = move_up { points.swap(index, index - 1); changed = true; }
                                                        if let Some(index) = move_down { points.swap(index, index + 1); changed = true; }
                                                        if ui.small_button("Add waypoint").clicked() { points.push(Value::Array(vec![Value::Number(serde_json::Number::from(0)), Value::Number(serde_json::Number::from(0))])); changed = true; }
                                                        if changed {
                                                            if apply_to_staged_entity_type(
                                                                document.as_deref_mut(),
                                                                hitbox_editor,
                                                                selected_name,
                                                                fallback_entity_type,
                                                                |et| { et.set_component_attribute_value(component_name, &row.name, Value::Array(points.clone())) },
                                                            ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                        }
                                                    });
                                                }
                                                attr if attr.starts_with("array") => {
                                                    let mut values = explicit_value.as_ref().and_then(|v| v.as_array().cloned()).or_else(|| display_default.as_ref().and_then(|v| v.as_array().cloned())).unwrap_or_default();
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
                                                                        if let Some(number) = serde_json::Number::from_f64(num) { *value = Value::Number(number); changed = true; }
                                                                    }
                                                                } else {
                                                                    let mut text = value.as_str().unwrap_or("").to_string();
                                                                    if ui.add(egui::TextEdit::singleline(&mut text).desired_width(middle_col_w)).changed() { *value = Value::String(text); changed = true; }
                                                                }
                                                                if ui.small_button("↑").clicked() && index > 0 { move_up = Some(index); }
                                                                if ui.small_button("↓").clicked() && index + 1 < len { move_down = Some(index); }
                                                                if ui.small_button("-").clicked() { remove_index = Some(index); }
                                                            });
                                                        }
                                                        if let Some(index) = remove_index { values.remove(index); changed = true; }
                                                        if let Some(index) = move_up { values.swap(index, index - 1); changed = true; }
                                                        if let Some(index) = move_down { values.swap(index, index + 1); changed = true; }
                                                        if ui.button(egui_phosphor_icons::icons::PLUS).clicked() { if is_number_array { values.push(Value::Number(serde_json::Number::from(0))); } else { values.push(Value::String(String::new())); } changed = true; }
                                                        if changed {
                                                            if apply_to_staged_entity_type(
                                                                document.as_deref_mut(),
                                                                hitbox_editor,
                                                                selected_name,
                                                                fallback_entity_type,
                                                                |et| { et.set_component_attribute_value(component_name, &row.name, Value::Array(values.clone())) },
                                                            ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                        }
                                                    });
                                                }
                                                _ => {
                                                    let editor_key = format!("json::{}::{}::{}", selected_name, component_name, row.name);
                                                    let initial_text = explicit_value.as_ref().or(display_default.as_ref()).map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string())).unwrap_or_else(|| "null".to_string());
                                                    let mut text_value = hitbox_editor.json_editor_state.get(&editor_key).cloned().unwrap_or(initial_text);
                                                    let response = ui.add(
                                                        egui::TextEdit::multiline(&mut text_value)
                                                            .id_salt(editor_key.clone())
                                                            // Fill the column without pushing it wider.
                                                            .desired_width(middle_col_w)
                                                            .desired_rows(3),
                                                    );
                                                    if response.changed() {
                                                        hitbox_editor.json_editor_state.insert(editor_key.clone(), text_value.clone());
                                                        if let Ok(parsed) = serde_json::from_str::<Value>(&text_value) {
                                                            if apply_to_staged_entity_type(
                                                                document.as_deref_mut(),
                                                                hitbox_editor,
                                                                selected_name,
                                                                fallback_entity_type,
                                                                |et| { et.set_component_attribute_value(component_name, &row.name, parsed) },
                                                            ) { hitbox_editor.dirty_entity_types.insert(selected_name.to_string()); }
                                                        }
                                                    }
                                                }
                                            }

                                            // restore visuals for column 2
                                            if !is_explicit { ui.visuals_mut().override_text_color = saved_override; }
                                        });

                                        // Column 3: clear/reset button (right-aligned with padding)
                                        r.col(|ui| {
                                            if explicit_value.is_some() {
                                                // Reserve the full column cell so layout stays consistent
                                                let cell_size = egui::vec2(clear_col_w, 20.0);
                                                let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
                                                 // center the reset button inside the clear column cell so it
                                                 // aligns visually with the header trash button above.
                                                 // Apply a small rightward nudge to account for slight
                                                 // visual differences in the icon glyphs so the perceived
                                                 // alignment matches the header button.
                                                 let btn_center_x = (cell_rect.min.x + cell_rect.max.x) * 0.5;
                                                 let align_nudge = 3.0; // tweak this if you prefer different visual alignment
                                                 let btn_rect = egui::Rect::from_min_max(
                                                     egui::pos2(btn_center_x - button_w * 0.5 + align_nudge, cell_rect.min.y),
                                                     egui::pos2(btn_center_x + button_w * 0.5 + align_nudge, cell_rect.max.y),
                                                 );
                                                 let reset_resp = ui.put(btn_rect, egui::Button::new(egui_phosphor_icons::icons::ARROW_COUNTER_CLOCKWISE).min_size(egui::vec2(button_w, 20.0)));
                                                reset_resp.clone().on_hover_text("Reset to default (removes explicit override from JSON)");
                                                if reset_resp.clicked() {
                                                    if apply_to_staged_entity_type(
                                                        document.as_deref_mut(),
                                                        hitbox_editor,
                                                        selected_name,
                                                        fallback_entity_type,
                                                        |et| et.remove_component_attribute(component_name, &row.name),
                                                    ) {
                                                        hitbox_editor.dirty_entity_types.insert(selected_name.to_string());
                                                        explicit_value = None;
                                                        let editor_key = format!("json::{}::{}::{}", selected_name, component_name, row.name);
                                                        hitbox_editor.json_editor_state.remove(&editor_key);
                                                    }
                                                }
                                            } else {
                                                ui.label("");
                                            }
                                        });
                                    });

                                    // Optional muted "default" hint row when no explicit value
                                    if explicit_value.is_none() {
                                        let hint = if component_default.is_some() { Some("component default") } else if enum_default.is_some() { Some("first enum option") } else { None };
                                        if let Some(source) = hint {
                                            body.row(20.0, |mut rr| {
                                                rr.col(|ui| { ui.label(""); });
                                                rr.col(|ui| { ui.label(egui::RichText::new("default").weak().italics()).on_hover_text(source); });
                                                rr.col(|ui| { ui.label(""); });
                                            });
                                        }
                                    }
                                }
                            });
                    }

                    ui.separator();
                    });
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
    active_character: Res<crate::editor::ActiveCharacter>,
    mapping: Res<crate::editor::ComponentValueMapping>,
    time: Res<Time>,
    mut toast: ResMut<crate::editor::ToastState>,
    mut widths: ResMut<crate::editor::table_ui::ColumnWidths>,
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
            // Resolve using active character: if original file missing, try suffixed variant.
            let resolved = {
                let fs_exact = crate::io::assets_dir().join(normalized);
                if fs_exact.exists() {
                    normalized.clone()
                } else if let Some(pos) = normalized.rfind('.') {
                    let (before_ext, ext) = normalized.split_at(pos);
                    if before_ext.ends_with(".bob") || before_ext.ends_with(".betty") {
                        normalized.clone()
                    } else {
                        let suf = match *active_character {
                            crate::editor::ActiveCharacter::Betty => "betty",
                            _ => "bob",
                        };
                        let suffixed = format!("{}.{suf}{}", before_ext, ext);
                        let fs_suff = crate::io::assets_dir().join(&suffixed);
                        if fs_suff.exists() { suffixed } else { normalized.clone() }
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
                let fs_resolved = crate::io::asset_path_to_filesystem_path(&resolved);
                if let Ok((w, h)) = image::image_dimensions(&fs_resolved) {
                    loaded_image_sizes.insert(normalized.clone(), (w, h));
                }
            }
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
                if ui.button(egui_phosphor_icons::icons::X).clicked() {
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
                        .id_salt(format!("entity_type_state_header_{}_{}", selected_name, state_key))
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
                                    // Use the image's display size as the frame size so the
                                    // drawn border exactly matches the sprite preview size.
                                    // Fall back to the preview canvas when dimensions are
                                    // unavailable (best-effort).
                                    let frame_size = display_size;
                                    let (canvas_rect, mut response) = ui.allocate_exact_size(
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
