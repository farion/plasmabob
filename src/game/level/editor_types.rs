use std::collections::{BTreeMap, HashMap};

use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

const DEFAULT_ENTITY_TYPES_PATH: &str = "entity_types";

fn default_entity_types_path() -> String {
    DEFAULT_ENTITY_TYPES_PATH.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LevelFile {
    #[serde(default)]
    pub terrain: Option<TerrainDefinition>,
    #[serde(default)]
    pub quotes: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_opt_vec_string_or_seq")]
    pub music: Option<Vec<String>>,
    #[serde(default)]
    pub bounds: Option<LevelBoundsDefinition>,
    #[serde(default = "default_entity_types_path")]
    pub entity_types_path: String,
    #[serde(default)]
    pub entities: Vec<EntityDefinition>,
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerrainDefinition {
    #[serde(default)]
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelBoundsDefinition {
    pub width: f32,
    pub height: f32,
}

impl LevelBoundsDefinition {
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityTypeStateDefinition {
    #[serde(default)]
    pub animation: Vec<String>,
    #[serde(default)]
    pub collider_box: Option<Vec<[f32; 2]>>,
    #[serde(default)]
    pub animation_frame_ms: Option<u64>,
    #[serde(default)]
    pub lock_ms: u64,
    #[serde(default)]
    pub sound_start: Option<String>,
    #[serde(default)]
    pub sound_loop: Option<String>,
    #[serde(default)]
    pub sound_end: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl EntityTypeStateDefinition {
    pub fn hitbox_points(&self) -> &[[f32; 2]] {
        self.collider_box.as_deref().unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateMachineDefinition {
    #[serde(default)]
    pub initial_state: String,
    #[serde(default)]
    pub states: BTreeMap<String, EntityTypeStateDefinition>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityTypeDefinition {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub category_tag: Option<String>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub components: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl EntityTypeDefinition {
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width.unwrap_or_default(), self.height.unwrap_or_default())
    }

    pub fn component_names(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }

    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    pub fn set_component_names(&mut self, names: &[String]) {
        let wanted: std::collections::HashSet<&str> = names.iter().map(String::as_str).collect();
        self.components.retain(|name, _| wanted.contains(name.as_str()));
        for name in names {
            self.components
                .entry(name.clone())
                .or_insert_with(|| Value::Object(Map::new()));
        }
    }

    pub fn component_attribute_value(&self, component: &str, attribute: &str) -> Option<&Value> {
        self.components.get(component)?.as_object()?.get(attribute)
    }

    pub fn state_machine(&self) -> Option<StateMachineDefinition> {
        serde_json::from_value(self.components.get("state_machine")?.clone()).ok()
    }

    pub fn set_state_machine(&mut self, state_machine: StateMachineDefinition) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(state_machine)?;
        self.components.insert("state_machine".to_string(), value);
        Ok(())
    }

    pub fn default_texture_asset_path(&self) -> Option<String> {
        let state_machine = self.state_machine()?;
        if !state_machine.initial_state.is_empty() {
            if let Some(path) = state_machine
                .states
                .get(&state_machine.initial_state)
                .and_then(|state| state.animation.first())
            {
                return Some(normalize_asset_reference(path));
            }
        }

        state_machine
            .states
            .values()
            .flat_map(|state| state.animation.iter())
            .next()
            .map(|path| normalize_asset_reference(path))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityDefinition {
    pub id: String,
    pub entity_type: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub z_index: Option<f32>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub components: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl EntityDefinition {
    pub fn component_attribute_value(&self, component: &str, attribute: &str) -> Option<&Value> {
        self.components.as_ref()?.get(component)?.as_object()?.get(attribute)
    }

    pub fn set_component_attribute_value(&mut self, component: &str, attribute: &str, value: Value) {
        let components = self.components.get_or_insert_with(Map::new);
        let component_value = components
            .entry(component.to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        let component_object = if let Value::Object(object) = component_value {
            object
        } else {
            *component_value = Value::Object(Map::new());
            component_value.as_object_mut().expect("component value must be an object")
        };

        component_object.insert(attribute.to_string(), value);
    }

    pub fn remove_component_attribute(&mut self, component: &str, attribute: &str) {
        let Some(components) = self.components.as_mut() else {
            return;
        };
        let Some(component_value) = components.get_mut(component) else {
            return;
        };
        let Some(component_object) = component_value.as_object_mut() else {
            return;
        };

        component_object.remove(attribute);
        if component_object.is_empty() {
            components.remove(component);
        }
        if components.is_empty() {
            self.components = None;
        }
    }
}

pub fn normalize_asset_reference(reference: &str) -> String {
    reference.trim().trim_start_matches("assets/").to_string()
}

fn deserialize_opt_vec_string_or_seq<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => Ok(Some(vec![s])),
        serde_json::Value::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                match item {
                    serde_json::Value::String(s) => out.push(s),
                    other => {
                        return Err(serde::de::Error::custom(format!(
                            "expected array of strings, got {other}"
                        )))
                    }
                }
            }
            Ok(Some(out))
        }
        other => Err(serde::de::Error::custom(format!(
            "expected string or array of strings, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_assets_prefix() {
        assert_eq!(normalize_asset_reference("assets/levels/level1.json"), "levels/level1.json");
        assert_eq!(normalize_asset_reference("entity_types/bob.json"), "entity_types/bob.json");
    }

    #[test]
    fn default_texture_uses_initial_state() {
        let entity_type: EntityTypeDefinition = serde_json::from_value(serde_json::json!({
            "width": 10,
            "height": 20,
            "components": {
                "state_machine": {
                    "initial_state": "idle",
                    "states": {
                        "idle": {
                            "animation": ["assets/bob/bob-default.png"]
                        }
                    }
                }
            }
        }))
        .expect("entity type should deserialize");

        assert_eq!(
            entity_type.default_texture_asset_path().as_deref(),
            Some("bob/bob-default.png")
        );
    }
}

