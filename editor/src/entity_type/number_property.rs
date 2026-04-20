use crate::entity_type::helpers::apply_to_staged_entity_type;
use crate::entity_type::helpers::AttributeUiRow;
use crate::entity_type::hitbox::EntityTypeEditorState;
use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_number_property(
    ui: &mut egui::Ui,
    row: &AttributeUiRow,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let is_int = row.attr_type == "int";
    let mut value_f = explicit_value
        .and_then(|v| v.as_f64())
        .or_else(|| display_default.and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    if ui
        .add(egui::DragValue::new(&mut value_f).speed(1.0))
        .changed()
    {
        if is_int {
            let int_val = value_f.round() as i64;
            let num = serde_json::Number::from(int_val);
            if crate::entity_type::apply_to_staged_entity_type(
                document,
                entity_type_editor,
                selected_name,
                fallback_entity_type,
                |et| {
                    et.set_component_attribute_value(component_name, &row.name, Value::Number(num))
                },
            ) {
                entity_type_editor
                    .dirty_entity_types
                    .insert(selected_name.to_string());
            }
        } else if let Some(numf) = serde_json::Number::from_f64(value_f) {
            if crate::entity_type::apply_to_staged_entity_type(
                document,
                entity_type_editor,
                selected_name,
                fallback_entity_type,
                |et| {
                    et.set_component_attribute_value(component_name, &row.name, Value::Number(numf))
                },
            ) {
                entity_type_editor
                    .dirty_entity_types
                    .insert(selected_name.to_string());
            }
        }
    }
}
