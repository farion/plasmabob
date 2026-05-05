use bevy::prelude::*;
use bevy_egui::egui;
use serde_json::Value;

use super::layout::TableLayout;

pub(super) fn render_attribute_name_cell(
    ui: &mut egui::Ui,
    attr_name: &str,
    name_col_w: f32,
    header_label_offset: f32,
) {
    let cell_size = egui::vec2(name_col_w, 20.0);
    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
    let label_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.min.x + header_label_offset, cell_rect.min.y),
        cell_rect.max,
    );
    ui.allocate_ui_at_rect(label_rect, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(attr_name);
        });
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_attribute_value_cell(
    ui: &mut egui::Ui,
    row: &crate::entity_type::AttributeUiRow,
    explicit_value: &Option<Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let is_explicit = explicit_value.is_some();
    let saved_override = ui.visuals().override_text_color;
    if !is_explicit {
        ui.visuals_mut().override_text_color = Some(egui::Color32::from_gray(140));
    }

    match row.attr_type.as_str() {
        "number" | "int" => {
            crate::core::components_overrides::render_number_property(
                ui,
                row,
                explicit_value.as_ref(),
                display_default,
                middle_col_w,
                component_name,
                selected_name,
                document.as_deref_mut(),
                entity_type_editor,
                fallback_entity_type,
            );
        }
        "string" => {
            crate::core::components_overrides::render_string_property(
                ui,
                explicit_value.as_ref(),
                display_default,
                middle_col_w,
                component_name,
                selected_name,
                document.as_deref_mut(),
                entity_type_editor,
                fallback_entity_type,
                &row.name,
            );
        }
        "bool" => {
            crate::core::components_overrides::render_bool_property(
                ui,
                explicit_value.as_ref(),
                display_default,
                component_name,
                selected_name,
                document.as_deref_mut(),
                entity_type_editor,
                fallback_entity_type,
                &row.name,
            );
        }
        "enum" => {
            crate::core::components_overrides::render_enum_property(
                ui,
                row,
                explicit_value.as_ref(),
                display_default,
                component_name,
                selected_name,
                document.as_deref_mut(),
                entity_type_editor,
                fallback_entity_type,
            );
        }
        attr if attr.starts_with("array") => {
            crate::core::components_overrides::render_array_property(
                ui,
                &row.name,
                &row.attr_type,
                explicit_value.as_ref(),
                display_default,
                middle_col_w,
                component_name,
                selected_name,
                document.as_deref_mut(),
                entity_type_editor,
                fallback_entity_type,
            );
        }
        _ => {
            render_json_value_cell(
                ui,
                explicit_value.as_ref(),
                display_default,
                middle_col_w,
                component_name,
                &row.name,
                selected_name,
                document,
                entity_type_editor,
                fallback_entity_type,
            );
        }
    }

    if !is_explicit {
        ui.visuals_mut().override_text_color = saved_override;
    }
}

#[allow(clippy::too_many_arguments)]
fn render_json_value_cell(
    ui: &mut egui::Ui,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    attr_name: &str,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let editor_key = format!("json::{}::{}::{}", selected_name, component_name, attr_name);
    let initial_text = explicit_value
        .or(display_default)
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());
    let btn_w = 26.0_f32;
    let cell_size = egui::vec2(middle_col_w, 20.0);
    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());

    let btn_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.min.x, cell_rect.min.y),
        egui::pos2(cell_rect.min.x + btn_w, cell_rect.max.y),
    );
    let btn_resp = ui.put(
        btn_rect,
        egui::Button::new(egui_phosphor_icons::icons::PENCIL_SIMPLE)
            .min_size(egui::vec2(btn_w, 20.0)),
    );
    if btn_resp.clicked() {
        entity_type_editor
            .json_editor_state
            .entry(editor_key.clone())
            .or_insert_with(|| initial_text.clone());
    }

    let text_rect = egui::Rect::from_min_max(
        egui::pos2(btn_rect.max.x + 6.0, cell_rect.min.y),
        cell_rect.max,
    );
    ui.allocate_ui_at_rect(text_rect, |ui| {
        let text_w = text_rect.width().max(40.0);
        crate::core::components_overrides::render_json_property(
            ui,
            &editor_key,
            &initial_text,
            text_w,
            document.as_deref_mut(),
            entity_type_editor,
            selected_name,
            component_name,
            attr_name,
            fallback_entity_type,
        );
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_attribute_reset_cell(
    ui: &mut egui::Ui,
    has_explicit_value: bool,
    layout: &TableLayout,
    component_name: &str,
    attr_name: &str,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    if has_explicit_value {
        let cell_size = egui::vec2(layout.clear_col_w, 20.0);
        let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
        let btn_center_x = (cell_rect.min.x + cell_rect.max.x) * 0.5;
        let align_nudge = 3.0;
        let btn_rect = egui::Rect::from_min_max(
            egui::pos2(
                btn_center_x - layout.button_w * 0.5 + align_nudge,
                cell_rect.min.y,
            ),
            egui::pos2(
                btn_center_x + layout.button_w * 0.5 + align_nudge,
                cell_rect.max.y,
            ),
        );
        let reset_resp = ui.put(
            btn_rect,
            egui::Button::new(egui_phosphor_icons::icons::ARROW_COUNTER_CLOCKWISE)
                .min_size(egui::vec2(layout.button_w, 20.0)),
        );
        reset_resp
            .clone()
            .on_hover_text("Reset to default (removes explicit override from JSON)");
        if reset_resp.clicked() {
            if crate::entity_type::apply_to_staged_entity_type(
                document.as_deref_mut(),
                entity_type_editor,
                selected_name,
                fallback_entity_type,
                |et| et.remove_component_attribute(component_name, attr_name),
            ) {
                entity_type_editor
                    .dirty_entity_types
                    .insert(selected_name.to_string());
                let editor_key =
                    format!("json::{}::{}::{}", selected_name, component_name, attr_name);
                entity_type_editor.json_editor_state.remove(&editor_key);
            }
        }
    } else {
        ui.label("");
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_component_array_modal(
    ctx: &egui::Context,
    component_name: &str,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    toast: &mut crate::level::state::ToastState,
    time: &Time,
) {
    if let Some(editor) = entity_type_editor.array_editor.as_mut() {
        if editor.component_name == component_name {
            let (commit_values, commit_target, commit_close) =
                crate::core::components_overrides::render_array_modal(ctx, editor, toast, time);

            if let Some(vals) = commit_values {
                if let Some((comp, attr)) = commit_target {
                    if crate::entity_type::apply_to_staged_entity_type(
                        document.as_deref_mut(),
                        entity_type_editor,
                        selected_name,
                        fallback_entity_type,
                        |et| {
                            et.set_component_attribute_value(
                                &comp,
                                &attr,
                                Value::Array(vals.clone()),
                            )
                        },
                    ) {
                        entity_type_editor
                            .dirty_entity_types
                            .insert(selected_name.to_string());
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
}
