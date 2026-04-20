use super::array_editor::{
    format_array_short, inner_array_value_to_csv_string, parse_array_type_signature,
    ArrayEditorState,
};
use crate::entity_type::hitbox::EntityTypeEditorState;
use bevy_egui::egui;
use serde_json::Value;

// Renders the compact sidebar view for an attribute whose type starts with "array".
// Shows a short representation and a pencil button that initializes the array
// editor state stored in entity_type_editor.array_editor.
pub(crate) fn render_array_property(
    ui: &mut egui::Ui,
    row_name: &str,
    attr_type: &str,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let mut values = explicit_value
        .and_then(|v| v.as_array().cloned())
        .or_else(|| display_default.and_then(|v| v.as_array().cloned()))
        .unwrap_or_default();

    // compute a short JSON-like repr using helper from array_editor
    let short = format_array_short(&values);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(short));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new(egui_phosphor_icons::icons::PENCIL_SIMPLE))
                .clicked()
            {
                // initialize array editor state
                let type_desc = format!("{}.{} {}", component_name, row_name, attr_type);
                let parsed = parse_array_type_signature(attr_type);
                let mut inner_edit_strings = Vec::new();
                for v in &values {
                    if parsed.element_is_array {
                        inner_edit_strings.push(inner_array_value_to_csv_string(v));
                    } else {
                        inner_edit_strings.push(match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            other => serde_json::to_string(other).unwrap_or_default(),
                        });
                    }
                }

                entity_type_editor.array_editor = Some(ArrayEditorState {
                    component_name: component_name.to_string(),
                    attr_name: row_name.to_string(),
                    display_type: type_desc,
                    values: values.clone(),
                    original: values.clone(),
                    element_is_array: parsed.element_is_array,
                    element_is_number: parsed.element_is_number,
                    inner_fixed_len: parsed.inner_fixed_len,
                    outer_fixed_len: parsed.outer_fixed_len,
                    inner_edit_strings,
                    modal_pos: egui::pos2(0.0, 0.0),
                    modal_size: egui::vec2(500.0, 300.0),
                    modal_initialized: false,
                });
            }
        });
    });
}
