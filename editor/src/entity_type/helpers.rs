use bevy::prelude::*;
use serde_json::Value;
use std::collections::HashSet;

use crate::entity_type::EntityTypeEditorState;

/// Lightweight row description used by the attribute table renderers.
#[derive(Clone)]
pub(crate) struct AttributeUiRow {
    pub(crate) name: String,
    pub(crate) attr_type: String,
    pub(crate) options: Vec<String>,
}

pub(crate) fn cloned_staged_entity_type(
    document: Option<&crate::level::state::EditorDocument>,
    entity_type_editor: &EntityTypeEditorState,
    selected_name: &str,
    fallback: &crate::core::EntityTypeDefinition,
) -> crate::core::EntityTypeDefinition {
    if let Some(doc) = document {
        return doc
            .entity_types
            .get(selected_name)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
    }

    entity_type_editor
        .edited_entity_types
        .get(selected_name)
        .cloned()
        .unwrap_or_else(|| fallback.clone())
}

pub(crate) fn apply_to_staged_entity_type(
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut EntityTypeEditorState,
    selected_name: &str,
    fallback: &crate::core::EntityTypeDefinition,
    mutator: impl FnOnce(&mut crate::core::EntityTypeDefinition),
) -> bool {
    if let Some(doc) = document {
        if let Some(et) = doc.entity_types.get_mut(selected_name) {
            mutator(et);
            return true;
        }
        return false;
    }

    let et = entity_type_editor
        .edited_entity_types
        .entry(selected_name.to_string())
        .or_insert_with(|| fallback.clone());
    mutator(et);
    true
}

pub(crate) fn component_object_snapshot(
    entity_type: &crate::core::EntityTypeDefinition,
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

pub(crate) fn component_default_value(component_name: &str, attribute_name: &str) -> Option<Value> {
    let mut probe = crate::core::EntityTypeDefinition {
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

pub(crate) fn save_staged_entity_type(
    document: Option<&crate::level::state::EditorDocument>,
    entity_type_editor: &EntityTypeEditorState,
    selected_name: &str,
    fallback: &crate::core::EntityTypeDefinition,
) -> Result<(), String> {
    let mut to_save =
        cloned_staged_entity_type(document, entity_type_editor, selected_name, fallback);

    if !entity_type_editor.dirty_states.is_empty() {
        let mut state_machine = to_save
            .state_machine()
            .ok_or_else(|| "Cannot save hitboxes: missing state_machine component".to_string())?;

        for state_key in &entity_type_editor.dirty_states {
            if let Some(rect) = entity_type_editor.edited_hitboxes.get(state_key) {
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

    crate::core::io::save_entity_type_definition(selected_name, &to_save)
}

pub(crate) fn sorted_attribute_rows(
    mapping: &crate::level::state::ComponentValueMapping,
    entity_type: &crate::core::EntityTypeDefinition,
    fallback_entity: &crate::core::EntityTypeDefinition,
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
                let allow_extra =
                    matches!(component_name.to_ascii_lowercase().as_str(), "collider");
                if allow_extra || super::component_attribute_type(component_name, name).is_some() {
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
        if let Some(attr_type) = super::component_attribute_type(component_name, &key) {
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
    for &(name, typ) in super::component_declared_attributes(component_name) {
        if !seen.contains(&name.to_string()) {
            seen.insert(name.to_string());
            rows.push(AttributeUiRow {
                name: name.to_string(),
                attr_type: typ.to_string(),
                options: Vec::new(),
            });
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
