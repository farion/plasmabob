use std::collections::{HashMap, HashSet};

use bevy::prelude::Time;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use serde_json::Value;

#[derive(Default)]
pub(crate) struct LevelOverrideEdits {
    pub updates: HashMap<String, Value>,
    pub removals: HashSet<String>,
}

impl LevelOverrideEdits {
    pub fn has_changes(&self) -> bool {
        !self.updates.is_empty() || !self.removals.is_empty()
    }

    fn set(&mut self, key: String, value: Value) {
        self.removals.remove(&key);
        self.updates.insert(key, value);
    }

    fn remove(&mut self, key: String) {
        self.updates.remove(&key);
        self.removals.insert(key);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_level_entity_overrides_table(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    entity_id: &str,
    entity_type_name: &str,
    current_overrides: &HashMap<String, Value>,
    entity_type: &crate::core::EntityTypeDefinition,
    mapping: &crate::level::state::ComponentValueMapping,
    collapsed_components: &mut HashSet<String>,
    json_editor_state: &mut HashMap<String, String>,
    array_editor: &mut Option<crate::core::components_overrides::ArrayEditorState>,
    toast: &mut crate::level::state::ToastState,
    time: &Time,
    widths: &mut crate::core::ColumnWidths,
) -> LevelOverrideEdits {
    let mut edits = LevelOverrideEdits::default();
    let components_snapshot = entity_type.component_names();

    if components_snapshot.is_empty() {
        return edits;
    }

    ui.separator();
    ui.heading("Component Overrides");
    ui.label(format!("Entity: {} ({})", entity_id, entity_type_name));
    ui.add_space(4.0);

    let button_padding = 8.0f32;
    let button_w = 24.0f32;
    let clear_col_w = button_padding * 2.0 + button_w;
    let mut name_col_w_global = widths.widths.first().cloned().unwrap_or(80.0);
    let col_spacing = ui.spacing().item_spacing.x * 2.0 + 4.0;

    let mut max_name_chars: usize = 0;
    for comp in &components_snapshot {
        let rows =
            crate::entity_type::sorted_attribute_rows(mapping, entity_type, entity_type, comp);
        for r in rows {
            max_name_chars = max_name_chars.max(r.name.len());
        }
    }
    let ppp = ui.ctx().pixels_per_point();
    let avg_char_w = 7.0 * ppp;
    let desired_name_w = (max_name_chars as f32) * avg_char_w + 12.0;
    let max_allowed = (ui.available_width() - clear_col_w - col_spacing - 40.0).max(40.0);
    name_col_w_global = name_col_w_global.max(desired_name_w).min(max_allowed);
    let text_col_w_global =
        (ui.available_width() - name_col_w_global - clear_col_w - col_spacing).max(40.0);
    widths.widths = vec![name_col_w_global, text_col_w_global, clear_col_w];

    egui::ScrollArea::vertical()
        .id_salt(format!("level_components_scroll_{}", entity_id))
        .show(ui, |ui| {
            for component_name in &components_snapshot {
                let attr_rows =
                    crate::entity_type::sorted_attribute_rows(mapping, entity_type, entity_type, component_name);

                let component_scope_id = format!("level_component_section_{}_{}", entity_id, component_name);
                ui.push_id(component_scope_id, |ui| {
                    ui.add_space(4.0);
                    let header_h = 24.0f32;
                    let header_size = egui::vec2(ui.available_width(), header_h);
                    let (header_rect, _) = ui.allocate_exact_size(header_size, egui::Sense::click());

                    let is_collapsed = collapsed_components.contains(component_name);
                    let arrow_icon = if is_collapsed {
                        egui_phosphor_icons::icons::CARET_RIGHT
                    } else {
                        egui_phosphor_icons::icons::CARET_DOWN
                    };
                    let arrow_rect = egui::Rect::from_min_max(
                        egui::pos2(header_rect.min.x + 4.0, header_rect.min.y),
                        egui::pos2(header_rect.min.x + 4.0 + button_w, header_rect.max.y),
                    );
                    if ui
                        .put(
                            arrow_rect,
                            egui::Button::new(arrow_icon).min_size(egui::vec2(button_w, header_h)),
                        )
                        .clicked()
                    {
                        if is_collapsed {
                            collapsed_components.remove(component_name);
                        } else {
                            collapsed_components.insert(component_name.clone());
                        }
                    }

                    let header_label_offset = arrow_rect.max.x - header_rect.min.x + 6.0;
                    let label_rect = egui::Rect::from_min_max(
                        egui::pos2(header_rect.min.x + header_label_offset, header_rect.min.y),
                        header_rect.max,
                    );
                    ui.allocate_ui_at_rect(label_rect, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(component_name).strong()));
                        });
                    });

                    if !collapsed_components.contains(component_name) {
                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::exact(name_col_w_global))
                            .column(Column::exact(text_col_w_global))
                            .column(Column::exact(clear_col_w));

                        table.body(|mut body| {
                            for row in &attr_rows {
                                let key = format!("{}.{}", component_name, row.name);
                                let explicit_value = current_overrides.get(&key);
                                // Treat explicit JSON null the same as absent so the
                                // editor does not show a Reset button or treat the
                                // field as an explicit override for null-valued
                                // attributes (they can appear when converting
                                // typed configs to raw JSON).
                                let is_explicit = explicit_value.map(|v| !v.is_null()).unwrap_or(false);
                                let enum_default = if row.attr_type == "enum" {
                                    row.options.first().cloned().map(Value::String)
                                } else {
                                    None
                                };
                                let display_default = entity_type
                                    .component_attribute_value(component_name, &row.name)
                                    .or_else(|| crate::entity_type::component_default_value(component_name, &row.name))
                                    .or(enum_default);

                                body.row(20.0, |mut r| {
                                    r.col(|ui| {
                                        // Match entity-type sidebar: reserve the same
                                        // left offset for the header arrow so attribute
                                        // names align under the component label.
                                        let cell_size = egui::vec2(name_col_w_global, 20.0);
                                        let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
                                        let label_rect = egui::Rect::from_min_max(
                                            egui::pos2(cell_rect.min.x + header_label_offset, cell_rect.min.y),
                                            cell_rect.max,
                                        );
                                        ui.allocate_ui_at_rect(label_rect, |ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.label(&row.name);
                                            });
                                        });
                                    });

                                    r.col(|ui| {
                                        let saved = ui.visuals().override_text_color;
                                        if !is_explicit {
                                            ui.visuals_mut().override_text_color =
                                                Some(egui::Color32::from_gray(140));
                                        }

                                        match row.attr_type.as_str() {
                                            "number" | "int" => {
                                                let mut value = explicit_value
                                                    .and_then(|v| v.as_f64())
                                                    .or_else(|| display_default.as_ref().and_then(|v| v.as_f64()))
                                                    .unwrap_or(0.0);
                                                if ui.add(egui::DragValue::new(&mut value).speed(1.0)).changed() {
                                                    if row.attr_type == "int" {
                                                        edits.set(
                                                            key.clone(),
                                                            Value::Number(serde_json::Number::from(value.round() as i64)),
                                                        );
                                                    } else if let Some(n) = serde_json::Number::from_f64(value) {
                                                        edits.set(key.clone(), Value::Number(n));
                                                    }
                                                }
                                            }
                                            "string" => {
                                                let mut text = explicit_value
                                                    .and_then(|v| v.as_str())
                                                    .or_else(|| display_default.as_ref().and_then(|v| v.as_str()))
                                                    .unwrap_or("")
                                                    .to_string();
                                                if ui.add(egui::TextEdit::singleline(&mut text)).changed() {
                                                    edits.set(key.clone(), Value::String(text));
                                                }
                                            }
                                            "bool" => {
                                                let mut b = explicit_value
                                                    .and_then(|v| v.as_bool())
                                                    .or_else(|| display_default.as_ref().and_then(|v| v.as_bool()))
                                                    .unwrap_or(false);
                                                if ui.add(egui::Checkbox::new(&mut b, "")).changed() {
                                                    edits.set(key.clone(), Value::Bool(b));
                                                }
                                            }
                                            "enum" => {
                                                let mut current = explicit_value
                                                    .and_then(|v| v.as_str())
                                                    .or_else(|| display_default.as_ref().and_then(|v| v.as_str()))
                                                    .unwrap_or("")
                                                    .to_string();
                                                let before = current.clone();
                                                egui::ComboBox::from_id_salt(format!("level_enum_{}_{}", entity_id, key))
                                                    .selected_text(if current.is_empty() { "select..." } else { &current })
                                                    .show_ui(ui, |ui| {
                                                        for option in &row.options {
                                                            ui.selectable_value(&mut current, option.clone(), option);
                                                        }
                                                    });
                                                if !current.is_empty() && current != before {
                                                    edits.set(key.clone(), Value::String(current));
                                                }
                                            }
                                            attr if attr.starts_with("array") => {
                                                let values = explicit_value
                                                    .and_then(|v| v.as_array().cloned())
                                                    .or_else(|| display_default.as_ref().and_then(|v| v.as_array().cloned()))
                                                    .unwrap_or_default();

                                                crate::core::components_overrides::render_array_edit_button_and_short(
                                                    ui,
                                                    component_name,
                                                    &row.name,
                                                    &row.attr_type,
                                                    &values,
                                                    text_col_w_global,
                                                    array_editor,
                                                );
                                            }
                                            _ => {
                                                let editor_key = format!("level_json::{}::{}", entity_id, key);
                                                let mut text_value = json_editor_state
                                                    .get(&editor_key)
                                                    .cloned()
                                                    .unwrap_or_else(|| {
                                                        display_default
                                                            .as_ref()
                                                            .or(explicit_value)
                                                            .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string()))
                                                            .unwrap_or_else(|| "null".to_string())
                                                    });
                                                if ui
                                                    .add(egui::TextEdit::singleline(&mut text_value))
                                                    .changed()
                                                {
                                                    json_editor_state.insert(editor_key, text_value.clone());
                                                    if let Ok(parsed) = serde_json::from_str::<Value>(&text_value) {
                                                        edits.set(key.clone(), parsed);
                                                    }
                                                }
                                            }
                                        }

                                        ui.visuals_mut().override_text_color = saved;
                                    });

                                    r.col(|ui| {
                                        if is_explicit {
                                            if ui
                                                .button(egui_phosphor_icons::icons::ARROW_COUNTER_CLOCKWISE)
                                                .on_hover_text("Reset to entity type value")
                                                .clicked()
                                            {
                                                edits.remove(key.clone());
                                            }
                                        }
                                    });
                                });
                            }
                        });
                    }

                    ui.separator();

                    if let Some(editor) = array_editor.as_mut() {
                        if editor.component_name == *component_name {
                            let (commit_values, commit_target, commit_close) =
                                crate::core::components_overrides::render_array_modal(
                                    ctx,
                                    editor,
                                    toast,
                                    time,
                                );
                            if let Some(vals) = commit_values {
                                if let Some((comp, attr)) = commit_target {
                                    edits.set(format!("{}.{}", comp, attr), Value::Array(vals));
                                }
                            }
                            if commit_close {
                                *array_editor = None;
                            }
                        }
                    }
                });
            }
        });

    edits
}
