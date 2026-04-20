use bevy::prelude::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use serde_json::Value;

use super::hitbox::EntityTypeEditorState;

// Render the components sidebar. This was extracted from the parent module and
// uses helper functions defined in the parent via `super::` to avoid
// duplicating logic.
pub(crate) fn render_components_sidebar(
    ctx: &egui::Context,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    mapping: &crate::level::state::ComponentValueMapping,
    mut toast: &mut crate::level::state::ToastState,
    time: &Time,
    mut widths: ResMut<crate::core::ColumnWidths>,
) {
    // Delegate to the original logic but qualify calls to parent helpers with
    // super:: so this module remains a thin extraction.

    egui::SidePanel::right("entity_type_components_sidebar")
        .resizable(true)
        .default_width(450.0)
        .min_width(300.0)
        .max_width(600.0)
        .show(ctx, |ui| {
            // Begin copy of original sidebar implementation; calls to helpers
            // are qualified with `super::` where necessary.
            ui.heading("Components");
            ui.add_space(6.0);

            let available_components = match crate::core::io::scan_game_components() {
                Ok(v) => v,
                Err(e) => {
                    toast.message = Some(format!("Could not scan components: {}", e));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    Vec::new()
                }
            };

            let staged_snapshot = super::helpers::cloned_staged_entity_type(
                document.as_deref(),
                entity_type_editor,
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
                let mut selected = entity_type_editor.add_selected.clone().unwrap_or_default();
                egui::ComboBox::from_id_salt(format!("add_component_cb_{}", selected_name))
                    .selected_text(if selected.is_empty() { "select..." } else { &selected })
                    .show_ui(ui, |ui| {
                        for option in &add_options {
                            ui.selectable_value(&mut selected, option.clone(), option);
                        }
                    });
                entity_type_editor.add_selected = if selected.is_empty() { None } else { Some(selected.clone()) };

                let add_enabled = entity_type_editor
                    .add_selected
                    .as_ref()
                    .map(|selection| add_options.iter().any(|option| option == selection))
                    .unwrap_or(false);

                if ui.add_enabled(add_enabled, egui::Button::new(egui_phosphor_icons::icons::PLUS)).clicked() {
                    if let Some(chosen) = entity_type_editor.add_selected.clone() {
                        let mut new_components = components_snapshot.clone();
                        if !new_components.iter().any(|component| component == &chosen) {
                            new_components.push(chosen.clone());
                            if super::helpers::apply_to_staged_entity_type(
                                document.as_deref_mut(),
                                entity_type_editor,
                                selected_name,
                                fallback_entity_type,
                                |et| et.set_component_names(&new_components),
                            ) {
                                entity_type_editor.dirty_entity_types.insert(selected_name.to_string());
                            }
                        }
                    }
                    entity_type_editor.add_selected = None;
                }
            });

            let button_padding = 8.0f32;
            let button_w = 24.0f32;
            let clear_col_w = button_padding * 2.0 + button_w;
            let mut name_col_w_global = widths.widths.get(0).cloned().unwrap_or(80.0);
            let name_indent = 6.0f32;
            let col_spacing = ui.spacing().item_spacing.x * 2.0 + 4.0;

            let mut max_name_chars: usize = 0;
            for comp in &components_snapshot {
                let rows = super::helpers::sorted_attribute_rows(mapping, &staged_snapshot, fallback_entity_type, comp);
                for r in rows {
                    max_name_chars = max_name_chars.max(r.name.len());
                }
            }
            let ppp = ui.ctx().pixels_per_point();
            let avg_char_w = 7.0 * ppp;
            let desired_name_w = (max_name_chars as f32) * avg_char_w + 12.0;
            let max_allowed = (ui.available_width() - clear_col_w - col_spacing - 40.0).max(40.0);
            name_col_w_global = name_col_w_global.max(desired_name_w).min(max_allowed);

            let text_col_w_global = (ui.available_width() - name_col_w_global - clear_col_w - col_spacing).max(40.0);
            widths.widths = vec![name_col_w_global, text_col_w_global, clear_col_w];

            egui::ScrollArea::vertical()
                .id_salt(format!("entity_type_components_scroll_{}", selected_name))
                .show(ui, |ui| {
                    for component_name in &components_snapshot {
                        let attr_rows = super::helpers::sorted_attribute_rows(mapping, &staged_snapshot, fallback_entity_type, component_name);

                        let component_scope_id = format!("entity_type_component_section_{}_{}", selected_name, component_name);
                        ui.push_id(component_scope_id, |ui| {
                            ui.add_space(4.0);
                            let header_h = 24.0f32;
                            let header_size = egui::vec2(ui.available_width(), header_h);
                            let (header_rect, _header_resp) = ui.allocate_exact_size(header_size, egui::Sense::click());

                             let is_collapsed = entity_type_editor.collapsed_components.contains(component_name);

                            let left_rect = egui::Rect::from_min_max(header_rect.min, egui::pos2(header_rect.max.x - clear_col_w, header_rect.max.y));
                            let arrow_icon = if is_collapsed { egui_phosphor_icons::icons::CARET_RIGHT } else { egui_phosphor_icons::icons::CARET_DOWN };
                            let arrow_rect = egui::Rect::from_min_max(
                                egui::pos2(left_rect.min.x + 4.0, left_rect.min.y),
                                egui::pos2(left_rect.min.x + 4.0 + button_w, left_rect.max.y),
                            );
                            let arrow_resp = ui.put(arrow_rect, egui::Button::new(arrow_icon).min_size(egui::vec2(button_w, header_h)));
                            if arrow_resp.clicked() {
                                 if is_collapsed { entity_type_editor.collapsed_components.remove(component_name); } else { entity_type_editor.collapsed_components.insert(component_name.clone()); }
                            }

                            let label_rect = egui::Rect::from_min_max(
                                egui::pos2(arrow_rect.max.x + 6.0, left_rect.min.y),
                                egui::pos2(left_rect.max.x, left_rect.max.y),
                            );
                            // Use an allocated UI region with a left-aligned layout so the
                            // component name is left-aligned within the header cell.
                            ui.allocate_ui_at_rect(label_rect, |ui| {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    ui.add(egui::Label::new(egui::RichText::new(component_name).strong()));
                                });
                            });

                            let right_rect = egui::Rect::from_min_max(
                                egui::pos2(header_rect.max.x - clear_col_w, header_rect.min.y),
                                header_rect.max,
                            );
                            let btn_center_x = (right_rect.min.x + right_rect.max.x) * 0.5;
                            let button_rect = egui::Rect::from_min_max(
                                egui::pos2(btn_center_x - button_w * 0.5, right_rect.min.y),
                                egui::pos2(btn_center_x + button_w * 0.5, right_rect.max.y),
                            );
                            let trash_resp = ui.put(button_rect, egui::Button::new(egui_phosphor_icons::icons::TRASH).min_size(egui::vec2(button_w, header_h)));

                            ui.add_space(6.0);
                            if trash_resp.clicked() {
                                 entity_type_editor.remove_component_confirm = Some(component_name.clone());
                            }

                            if !entity_type_editor.collapsed_components.contains(component_name) {
                                let name_col_w = name_col_w_global;
                                let middle_col_w = text_col_w_global;
                                widths.widths = vec![name_col_w, middle_col_w, clear_col_w];

                                let table = TableBuilder::new(ui).striped(true)
                                    .column(Column::exact(name_col_w))
                                    .column(Column::exact(middle_col_w))
                                    .column(Column::exact(clear_col_w));

                                table.body(|mut body| {
                                    for row in &attr_rows {
                                        let mut explicit_value = staged_snapshot
                                            .component_attribute_value(component_name, &row.name);
                                        let component_default = super::helpers::component_default_value(component_name, &row.name);
                                        let enum_default = if row.attr_type == "enum" {
                                            row.options.first().cloned().map(Value::String)
                                        } else {
                                            None
                                        };
                                        let display_default = component_default.clone().or(enum_default.clone());

                                        body.row(20.0, |mut r| {
                                            r.col(|ui| {
                                                let cell_size = egui::vec2(name_col_w, 20.0);
                                                let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
                                                let label_rect = egui::Rect::from_min_max(
                                                    egui::pos2(cell_rect.min.x + name_indent, cell_rect.min.y),
                                                    cell_rect.max,
                                                );
                                                 // Left-align attribute name within the name column.
                                                 ui.allocate_ui_at_rect(label_rect, |ui| {
                                                     ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                                         ui.label(&row.name);
                                                     });
                                                 });
                                            });

                                             r.col(|ui| {
                                                let is_explicit = explicit_value.is_some();
                                                let saved_override = ui.visuals().override_text_color;
                                                if !is_explicit { ui.visuals_mut().override_text_color = Some(egui::Color32::from_gray(140)); }

                                                match row.attr_type.as_str() {
                                                    "number" | "int" => {
                                                         super::number_property::render_number_property(ui, row, explicit_value.as_ref(), display_default.as_ref(), middle_col_w, component_name, selected_name, document.as_deref_mut(), entity_type_editor, fallback_entity_type);
                                                    }
                                                    "string" => {
                                                         super::string_property::render_string_property(ui, explicit_value.as_ref(), display_default.as_ref(), middle_col_w, component_name, selected_name, document.as_deref_mut(), entity_type_editor, fallback_entity_type, &row.name);
                                                    }
                                                    "bool" => {
                                                         super::bool_property::render_bool_property(ui, explicit_value.as_ref(), display_default.as_ref(), component_name, selected_name, document.as_deref_mut(), entity_type_editor, fallback_entity_type, &row.name);
                                                    }
                                                    "enum" => {
                                                         super::enum_property::render_enum_property(ui, row, explicit_value.as_ref(), display_default.as_ref(), component_name, selected_name, document.as_deref_mut(), entity_type_editor, fallback_entity_type);
                                                    }
                                                    attr if attr.starts_with("array") => {
                                                         super::array_property::render_array_property(ui, &row.name, &row.attr_type, explicit_value.as_ref(), display_default.as_ref(), middle_col_w, component_name, selected_name, document.as_deref_mut(), entity_type_editor, fallback_entity_type);
                                                    }
                                                    _ => {
                                                        let editor_key = format!("json::{}::{}::{}", selected_name, component_name, row.name);
                                                        let initial_text = explicit_value.as_ref().or(display_default.as_ref()).map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string())).unwrap_or_else(|| "null".to_string());
                                                         super::json_property::render_json_property(ui, &editor_key, &initial_text, middle_col_w, document.as_deref_mut(), entity_type_editor, selected_name, component_name, &row.name, fallback_entity_type);
                                                    }
                                                }

                                                if !is_explicit { ui.visuals_mut().override_text_color = saved_override; }
                                            });

                                            r.col(|ui| {
                                                if explicit_value.is_some() {
                                                    let cell_size = egui::vec2(clear_col_w, 20.0);
                                                    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
                                                    let btn_center_x = (cell_rect.min.x + cell_rect.max.x) * 0.5;
                                                    let align_nudge = 3.0;
                                                    let btn_rect = egui::Rect::from_min_max(
                                                        egui::pos2(btn_center_x - button_w * 0.5 + align_nudge, cell_rect.min.y),
                                                        egui::pos2(btn_center_x + button_w * 0.5 + align_nudge, cell_rect.max.y),
                                                    );
                                                    let reset_resp = ui.put(btn_rect, egui::Button::new(egui_phosphor_icons::icons::ARROW_COUNTER_CLOCKWISE).min_size(egui::vec2(button_w, 20.0)));
                                                    reset_resp.clone().on_hover_text("Reset to default (removes explicit override from JSON)");
                                                    if reset_resp.clicked() {
                                                        if super::helpers::apply_to_staged_entity_type(
                                                            document.as_deref_mut(),
                                                            entity_type_editor,
                                                            selected_name,
                                                            fallback_entity_type,
                                                            |et| et.remove_component_attribute(component_name, &row.name),
                                                        ) {
                                                            entity_type_editor.dirty_entity_types.insert(selected_name.to_string());
                                                            explicit_value = None;
                                                            let editor_key = format!("json::{}::{}::{}", selected_name, component_name, row.name);
                                                            entity_type_editor.json_editor_state.remove(&editor_key);
                                                        }
                                                    }
                                                } else {
                                                    ui.label("");
                                                }
                                            });
                                        });

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

                            if let Some(editor) = entity_type_editor.array_editor.as_mut() {
                                if editor.component_name == *component_name {
                                    let (commit_values, commit_target, commit_close) = crate::entity_type::array_editor::render_array_modal(ctx, editor, &mut toast, &time);

                                    if let Some(vals) = commit_values {
                                        if let Some((comp, attr)) = commit_target {
                                            if super::helpers::apply_to_staged_entity_type(
                                                document.as_deref_mut(),
                                                entity_type_editor,
                                                selected_name,
                                                fallback_entity_type,
                                                |et| et.set_component_attribute_value(&comp, &attr, Value::Array(vals.clone())),
                                            ) {
                                                entity_type_editor.dirty_entity_types.insert(selected_name.to_string());
                                            }
                                            if commit_close {
                                                entity_type_editor.array_editor = None;
                                            }
                                        }
                                    } else if commit_close {
                                        entity_type_editor.array_editor = None;
                                    }
                                }
                            }
                        });
                    }
                });
        });
}
