use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_string_property(
    ui: &mut egui::Ui,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    attr_name: &str,
) {
    let mut text = explicit_value
        .and_then(|v| v.as_str())
        .or_else(|| display_default.and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    if ui
        .add(egui::TextEdit::singleline(&mut text).desired_width(middle_col_w))
        .changed()
    {
        if crate::entity_type::apply_to_staged_entity_type(
            document,
            entity_type_editor,
            selected_name,
            fallback_entity_type,
            |et| et.set_component_attribute_value(component_name, attr_name, Value::String(text)),
        ) {
            entity_type_editor
                .dirty_entity_types
                .insert(selected_name.to_string());
        }
    }
}
