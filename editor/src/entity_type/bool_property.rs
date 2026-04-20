use crate::entity_type::helpers::apply_to_staged_entity_type;
use crate::entity_type::hitbox::EntityTypeEditorState;
use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_bool_property(
    ui: &mut egui::Ui,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    attr_name: &str,
) {
    let mut checked = explicit_value
        .and_then(|v| v.as_bool())
        .or_else(|| display_default.and_then(|v| v.as_bool()))
        .unwrap_or(false);
    let before = checked;
    if ui.add(egui::Checkbox::new(&mut checked, "")).changed() && checked != before {
        if apply_to_staged_entity_type(
            document,
            entity_type_editor,
            selected_name,
            fallback_entity_type,
            |et| et.set_component_attribute_value(component_name, attr_name, Value::Bool(checked)),
        ) {
            entity_type_editor
                .dirty_entity_types
                .insert(selected_name.to_string());
        }
    }
}
