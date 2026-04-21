use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_json_property(
    ui: &mut egui::Ui,
    editor_key: &str,
    initial_text: &str,
    middle_col_w: f32,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    selected_name: &str,
    component_name: &str,
    attr_name: &str,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let mut text_value = entity_type_editor
        .json_editor_state
        .get(editor_key)
        .cloned()
        .unwrap_or(initial_text.to_string());
    let response = ui.add(
        egui::TextEdit::multiline(&mut text_value)
            .id_salt(editor_key)
            .desired_width(middle_col_w)
            .desired_rows(3),
    );
    if response.changed() {
        entity_type_editor
            .json_editor_state
            .insert(editor_key.to_string(), text_value.clone());
        if let Ok(parsed) = serde_json::from_str::<Value>(&text_value) {
            if crate::entity_type::apply_to_staged_entity_type(
                document,
                entity_type_editor,
                selected_name,
                fallback_entity_type,
                |et| et.set_component_attribute_value(component_name, attr_name, parsed),
            ) {
                entity_type_editor
                    .dirty_entity_types
                    .insert(selected_name.to_string());
            }
        }
    }
}
