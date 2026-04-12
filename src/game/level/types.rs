use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::game::level::errors::LoadLevelError;

// ─── Level bounds ─────────────────────────────────────────────────────────────

/// World-space dimensions of a level (width × height in pixels/units).
#[derive(Debug, Clone, Deserialize)]
pub struct LevelBounds {
    pub width: f32,
    pub height: f32,
}

impl Default for LevelBounds {
    fn default() -> Self {
        LevelBounds { width: 4096.0, height: 1024.0 }
    }
}

// ─── State machine types ──────────────────────────────────────────────────────

/// Typed state machine configuration parsed from an entity type JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct StateMachineConfig {
    pub initial_state: String,
    #[serde(default)]
    pub states: HashMap<String, StateConfig>,
}

/// Configuration for a single animation state.
#[derive(Debug, Clone, Deserialize)]
pub struct StateConfig {
    /// Ordered list of sprite asset paths forming the animation.
    #[serde(default)]
    pub animation: Vec<String>,
    /// Duration of each frame in milliseconds.
    #[serde(default = "default_animation_frame_ms")]
    pub animation_frame_ms: u64,
    /// Collision box as a list of [x, y] pixel points in sprite-image space.
    #[serde(default)]
    pub collider_box: Option<Vec<[f32; 2]>>,
    /// Whether another state can interrupt this one before lock_ms elapses.
    #[serde(default)]
    pub interruptible: bool,
    /// Minimum time (ms) to stay in this state before transitioning.
    #[serde(default)]
    pub lock_ms: u64,
}

fn default_animation_frame_ms() -> u64 {
    180
}

// ─── Primitive property values ────────────────────────────────────────────────

/// Primitive property values parsed from level JSON. We avoid keeping raw
/// serde_json::Value in the runtime cache by converting to these typed
/// variants.
#[derive(Debug, Clone)]
pub enum PropValue {
    String(String),
    Number(f64),
    Bool(bool),
    Other(String), // fallback: serialized JSON for arrays/objects
}

impl From<serde_json::Value> for PropValue {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::String(s) => PropValue::String(s),
            serde_json::Value::Number(n) => PropValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => PropValue::Bool(b),
            other => PropValue::Other(other.to_string()),
        }
    }
}

/// Minimal runtime representation of a level file used by the Game view.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LevelDefinition {
    #[serde(default)]
    pub terrain: Option<TerrainDefinition>,
    #[serde(default, deserialize_with = "deserialize_opt_vec_string_or_seq")]
    pub music: Option<Vec<String>>,
    /// Path (asset) to the entity types configuration or directory. Optional.
    #[serde(default, rename = "entity_types_path")]
    pub entity_types_path: Option<String>,
    /// Entities defined in this level.
    #[serde(default)]
    pub entities: Option<Vec<LevelEntity>>,
    /// World bounds (width × height in world units).
    #[serde(default)]
    pub bounds: Option<LevelBounds>,
    /// Asset path to the level background image.
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TerrainDefinition {
    #[serde(default)]
    pub background: Option<String>,
}

/// Minimal runtime representation of an entity type JSON.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EntityTypeDefinition {
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub component: Vec<String>,
    /// Optional high-level category tag from the entity type JSON (e.g. "player", "enemy", "doodad").
    /// Prefer this over the first entry in `component` when present.
    #[serde(default)]
    pub category_tag: Option<String>,
    #[serde(default)]
    pub states: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    /// Maximum health points for this entity type.
    #[serde(default)]
    pub health: Option<u32>,
    /// Raw component configuration object (contains `state_machine`, `collider`, …).
    #[serde(default)]
    pub components: Option<serde_json::Value>,
}

impl EntityTypeDefinition {
    /// Extract and parse the typed state machine configuration from the raw
    /// `components.state_machine` field. Returns `None` when absent or malformed.
    pub fn state_machine_config(&self) -> Option<StateMachineConfig> {
        let components = self.components.as_ref()?;
        let sm_val = components.get("state_machine")?;
        serde_json::from_value(sm_val.clone()).ok()
    }
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::String(s) => Ok(vec![s]),
        serde_json::Value::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                if let serde_json::Value::String(s) = item {
                    out.push(s);
                } else {
                    return Err(serde::de::Error::custom("expected array of strings"));
                }
            }
            Ok(out)
        }
        _ => Err(serde::de::Error::custom("expected string or array of strings")),
    }
}

/// Entity instance parsed from the level JSON. Known fields are typed;
/// unknown additional properties are preserved in `properties` as
/// typed `PropValue`s (not raw JSON).
#[derive(Debug, Clone)]
pub(crate) struct LevelEntity {
    pub id: String,
    pub entity_type: String,
    pub x: f32,
    pub y: f32,
    pub z_index: f32,
    pub layer: String,
    pub properties: HashMap<String, PropValue>,
}

impl<'de> serde::Deserialize<'de> for LevelEntity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into an intermediate map so we can extract known fields
        let map: serde_json::Map<String, serde_json::Value> = serde::Deserialize::deserialize(deserializer)?;

        let id = map.get("id").and_then(|v| v.as_str()).ok_or_else(|| serde::de::Error::missing_field("id"))?.to_string();
        let entity_type = map.get("entity_type").and_then(|v| v.as_str()).ok_or_else(|| serde::de::Error::missing_field("entity_type"))?.to_string();
        let x = map.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = map.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let z_index = map.get("z_index").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("gameplay").to_string();

        let mut properties: HashMap<String, PropValue> = HashMap::new();
        for (k, v) in map.into_iter() {
            if k == "id" || k == "entity_type" || k == "x" || k == "y" || k == "z_index" || k == "layer" {
                continue;
            }
            properties.insert(k, PropValue::from(v));
        }

        Ok(LevelEntity {
            id,
            entity_type,
            x,
            y,
            z_index,
            layer,
            properties,
        })
    }
}

/// Cached representation of a loaded level. The GameView can insert this
/// Resource (or call `refresh`) to make a loaded level available to systems.
#[derive(Resource, Debug, Default)]
pub(crate) struct CachedLevelDefinition {
    pub asset_path: Option<String>,
    pub level: Option<LevelDefinition>,
    pub entity_types: HashMap<String, EntityTypeDefinition>,
    pub error: Option<LoadLevelError>,
}

// Note: the loader (in `loader.rs`) provides functions to produce a
// `CachedLevelDefinition`. We intentionally do not implement `refresh`
// here to avoid circular module dependencies; callers should use
// `crate::game::level::loader::load_level_from_asset` and then populate
// the `CachedLevelDefinition` resource as needed.

// --- Parsing helpers for unit tests ---
pub fn parse_level_definition(json: &str) -> Result<LevelDefinition, serde_json::Error> {
    serde_json::from_str(json)
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
                    other => return Err(serde::de::Error::custom(format!("expected array of strings, got {}", other))),
                }
            }
            Ok(Some(out))
        }
        other => Err(serde::de::Error::custom(format!("expected string or array of strings, got {}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_level() {
        let json = r#"{"entity_types_path":"entity_types","entities":[]}"#;
        let lvl = parse_level_definition(json).expect("should parse");
        assert!(lvl.entity_types_path.is_some());
    }

    #[test]
    fn parse_level_with_terrain() {
        let json = r#"{"terrain":{"background":"assets/backgrounds/level1.png"},"music":"assets/music/level1.ogg"}"#;
        let lvl = parse_level_definition(json).expect("should parse");
        assert_eq!(lvl.terrain.unwrap().background.unwrap(), "assets/backgrounds/level1.png");
    }

    #[test]
    fn load_level_and_entity_types_from_assets() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let level_path = std::path::Path::new(manifest_dir).join("assets/worlds/auralis/viridara_level1.json");
        let content = std::fs::read_to_string(&level_path).expect("should read level json");

        let lvl = parse_level_definition(&content).expect("should parse level json");
        let entities = lvl.entities.expect("entities present");
        assert!(entities.len() > 0, "expected some entities");

        // Load all entity type JSON files from assets/entity_types
        let et_dir = std::path::Path::new(manifest_dir).join("assets/entity_types");
        let mut et_map = std::collections::HashMap::<String, crate::game::level::types::EntityTypeDefinition>::new();
        for entry in std::fs::read_dir(&et_dir).expect("read entity_types dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let stem = path.file_stem().and_then(|s| s.to_str()).expect("stem").to_string();
            let txt = std::fs::read_to_string(&path).expect("read et file");
            let et: crate::game::level::types::EntityTypeDefinition = serde_json::from_str(&txt).expect("parse et json");
            et_map.insert(stem, et);
        }

        // Ensure every entity's entity_type has a corresponding entity type definition
        for e in &entities {
            assert!(et_map.contains_key(&e.entity_type), "missing entity type for {}", e.entity_type);
        }
    }
}


