use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_enum_property(
    ui: &mut egui::Ui,
    row: &crate::entity_type::AttributeUiRow,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::editor::EditorDocument>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::model::EntityTypeDefinition,
) {
    let mut current = explicit_value
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .or_else(|| display_default.and_then(|v| v.as_str()).map(|v| v.to_string()))
        .or_else(|| row.options.first().cloned())
        .unwrap_or_default();
    let before_current = current.clone();
    egui::ComboBox::from_id_salt(format!("entity_type_enum_{}_{}_{}", selected_name, component_name, row.name))
        .selected_text(if current.is_empty() { "select..." } else { &current })
        .show_ui(ui, |ui| { for option in &row.options { ui.selectable_value(&mut current, option.clone(), option); } });
    if !current.is_empty() && current != before_current {
        if crate::entity_type::apply_to_staged_entity_type(
            document,
            entity_type_editor,
            selected_name,
            fallback_entity_type,
            |et| { et.set_component_attribute_value(component_name, &row.name, Value::String(current.clone())) },
        ) { entity_type_editor.dirty_entity_types.insert(selected_name.to_string()); }
    }
}
