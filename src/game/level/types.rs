use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::game::level::errors::LoadLevelError;
use std::sync::OnceLock;
use crate::game::level::configs::{
    HealthConfig,
    ControlledMovementConfig,
    AutoMovementConfig,
    MovingPlatformConfig,
    RigidBodyConfig,
    GravityConfig,
    BlockingConfig,
    ControlledRangeAttackConfig,
    AutoRangeAttackConfig,
    AutoMeleeAttackConfig,
    ControlledMeleeAttackConfig,
    DamageableConfig,
    TeamConfig,
    OrientationConfig,
    ColliderConfig,
};
// StateMachineConfig is parsed as a typed config; we don't store a runtime
// StateMachine component in ComponentsDef anymore.

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
    /// Minimum time (ms) to stay in this state before transitioning.
    #[serde(default)]
    pub lock_ms: u64,
    /// Sound played once when entering this state.
    #[serde(default)]
    pub sound_start: Option<String>,
    /// Sound looped while this state is active (starts after sound_start ends).
    #[serde(default)]
    pub sound_loop: Option<String>,
    /// Sound played once when leaving this state.
    #[serde(default)]
    pub sound_end: Option<String>,
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
///
/// Note: older formats used a `component: ["Name"]` array. The current
/// format uses a `components: { "ComponentName": { ...attrs... }, ... }`
/// object where keys are component names and values are optional config
/// objects for that component.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EntityTypeDefinition {
    /// Typed component definitions parsed from the entity type JSON.
    ///
    /// Each known component is represented as an optional raw JSON value so
    /// that existing `override_from_json` helpers can consume them. Unknown
    /// component keys are preserved in `extra` via `flatten`.
    #[serde(default)]
    pub components: Option<ComponentsDef>,
    /// Optional high-level category tag from the entity type JSON (e.g. "player", "enemy", "doodad").
    #[serde(default)]
    pub category_tag: Option<String>,
    // `state_machine` config is read from the typed `components.state_machine`
    // (see ComponentsDef) and exposed via `state_machine_config()`; the
    // top-level `states` raw map is no longer stored here.
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    /// Maximum health points for this entity type.
    #[serde(default)]
    pub health: Option<u32>,
    /// The key/name used in the `entity_types` map. Not present in the JSON
    /// and injected by the loader when the definitions are loaded.
    #[serde(skip)]
    pub key: String,
}

impl EntityTypeDefinition {
    /// Extract and parse the typed state machine configuration from the raw
    /// `components.state_machine` is materialized into a runtime
    /// `StateMachineComponent` (see `ComponentsDef`). Convert that typed
    /// runtime component back into the configuration struct so callers can
    /// obtain the initial_state string and the per-state `StateConfig`s.
    pub fn state_machine_config(&self) -> Option<StateMachineConfig> {
        let comps = self.components.as_ref()?;
        let smc = comps.state_machine.as_ref()?;
        Some(smc.clone())
    }
}

/// Typed representation of the `components` object in an entity type JSON.
///
/// This structure holds actual runtime component instances (constructed
/// from defaults and then overridden with JSON when present). We implement
/// a custom `Deserialize` to accept the JSON `components` object and build
/// component instances by applying `override_from_json`.
#[derive(Debug, Clone, Default)]
pub struct ComponentsDef {
    pub health: Option<HealthConfig>,
    pub controlled_movement: Option<ControlledMovementConfig>,
    pub auto_movement: Option<AutoMovementConfig>,
    pub moving_platform: Option<MovingPlatformConfig>,
    pub rigid_body: Option<RigidBodyConfig>,
    pub gravity: Option<GravityConfig>,
    pub blocking: Option<BlockingConfig>,
    pub controlled_range_attack: Option<ControlledRangeAttackConfig>,
    pub auto_range_attack: Option<AutoRangeAttackConfig>,
    pub auto_melee_attack: Option<AutoMeleeAttackConfig>,
    pub controlled_melee_attack: Option<ControlledMeleeAttackConfig>,
    pub damageable: Option<DamageableConfig>,
    pub team: Option<TeamConfig>,
    pub orientation: Option<OrientationConfig>,
    pub state_machine: Option<StateMachineConfig>,
    pub collider: Option<ColliderConfig>,
}

impl<'de> serde::Deserialize<'de> for ComponentsDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let mut out = ComponentsDef::default();

        let map = match v {
            serde_json::Value::Object(m) => m,
            serde_json::Value::Null => return Ok(out),
            other => return Err(serde::de::Error::custom(format!("expected object for components, got {}", other))),
        };

        for (k, val) in map.into_iter() {
            match k.to_ascii_lowercase().as_str() {
                "health" => out.health = Some(serde_json::from_value::<HealthConfig>(val).map_err(serde::de::Error::custom)?),
                "controlledmovement" | "controlled_movement" => out.controlled_movement = Some(serde_json::from_value::<ControlledMovementConfig>(val).map_err(serde::de::Error::custom)?),
                "automovement" | "auto_movement" => out.auto_movement = Some(serde_json::from_value::<AutoMovementConfig>(val).map_err(serde::de::Error::custom)?),
                "movingplatform" | "moving_platform" => out.moving_platform = Some(serde_json::from_value::<MovingPlatformConfig>(val).map_err(serde::de::Error::custom)?),
                "rigidbody" | "rigid_body" => out.rigid_body = Some(serde_json::from_value::<RigidBodyConfig>(val).map_err(serde::de::Error::custom)?),
                "gravity" => out.gravity = Some(serde_json::from_value::<GravityConfig>(val).map_err(serde::de::Error::custom)?),
                "blocking" => out.blocking = Some(serde_json::from_value::<BlockingConfig>(val).map_err(serde::de::Error::custom)?),
                "controlled_range_attack" | "controlledrangeattack" => out.controlled_range_attack = Some(serde_json::from_value::<ControlledRangeAttackConfig>(val).map_err(serde::de::Error::custom)?),
                "auto_range_attack" | "autorangeattack" => out.auto_range_attack = Some(serde_json::from_value::<AutoRangeAttackConfig>(val).map_err(serde::de::Error::custom)?),
                "auto_melee_attack" | "automeleeattack" => out.auto_melee_attack = Some(serde_json::from_value::<AutoMeleeAttackConfig>(val).map_err(serde::de::Error::custom)?),
                "controlled_melee_attack" | "controlledmeleeattack" => out.controlled_melee_attack = Some(serde_json::from_value::<ControlledMeleeAttackConfig>(val).map_err(serde::de::Error::custom)?),
                "damageable" => out.damageable = Some(serde_json::from_value::<DamageableConfig>(val).map_err(serde::de::Error::custom)?),
                "team" => out.team = Some(serde_json::from_value::<TeamConfig>(val).map_err(serde::de::Error::custom)?),
                "orientation" => out.orientation = Some(serde_json::from_value::<OrientationConfig>(val).map_err(serde::de::Error::custom)?),
                "state_machine" | "statemachine" => out.state_machine = Some(serde_json::from_value::<StateMachineConfig>(val).map_err(serde::de::Error::custom)?),
                "collider" => out.collider = Some(serde_json::from_value::<ColliderConfig>(val).map_err(serde::de::Error::custom)?),
                other => {
                    // Unknown component keys are ignored for now; log to help
                    // designers discover typos in JSON.
                    tracing::warn!(comp = %other, "ComponentsDef: unknown component key in entity-type, ignoring");
                }
            }
        }

        Ok(out)
    }
}


/// Entity instance parsed from the level JSON. The `entity_type` field is
/// deserialized from a String key in the JSON and resolved via the global
/// entity-type registry into a typed `EntityTypeDefinition` instance. The
/// loader must register entity types before deserializing levels.
#[derive(Debug, Clone)]
pub(crate) struct LevelEntity {
    pub id: String,
    pub entity_type: EntityTypeDefinition,
    pub x: f32,
    pub y: f32,
    pub z_index: f32,
    pub name: String,
    pub layer: String,
    /// Optional per-level components overrides parsed from the level JSON's
    /// `components` object. This is a typed `ComponentsDef` (mirrors
    /// `EntityTypeDefinition.components`).
    pub components: Option<ComponentsDef>,
}

// Global registry for entity type definitions used during deserialization.
static ENTITY_TYPE_REGISTRY: OnceLock<HashMap<String, EntityTypeDefinition>> = OnceLock::new();

/// Register the entity types map for subsequent `LevelEntity` deserialization.
pub fn register_entity_types(map: HashMap<String, EntityTypeDefinition>) -> Result<(), String> {
    // If the registry is already populated, treat a re-registration as
    // idempotent when the same set of keys is being registered. This
    // prevents failing when the loader is invoked multiple times with the
    // same entity-types directory during development hot-reloads.
    if let Some(existing) = ENTITY_TYPE_REGISTRY.get() {
        // Quick sanity: if the incoming map has exactly the same keys as
        // the existing registry, consider this a no-op and return Ok.
        let same_keys = existing.len() == map.len() && existing.keys().all(|k| map.contains_key(k));
        if same_keys {
            return Ok(());
        }
        return Err("entity types already registered with different contents".to_string());
    }

    ENTITY_TYPE_REGISTRY
        .set(map)
        .map_err(|_| "entity types already registered".to_string())
}

fn get_registered_entity_type(name: &str) -> Option<&'static EntityTypeDefinition> {
    ENTITY_TYPE_REGISTRY.get().and_then(|m| m.get(name))
}

impl<'de> serde::Deserialize<'de> for LevelEntity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into an intermediate map so we can extract known fields
        let map: serde_json::Map<String, serde_json::Value> = serde::Deserialize::deserialize(deserializer)?;

        let id = map
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("id"))?
            .to_string();

        let et_key = map
            .get("entity_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("entity_type"))?
            .to_string();

        // Lookup entity type in the global registry populated by the loader.
        let et = get_registered_entity_type(&et_key)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown entity_type '{}' (registry not populated)", et_key)))?
            .clone();

        let x = map.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = map.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let z_index = map.get("z_index").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("gameplay").to_string();
        let name = map.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| id.clone());

        let mut components: Option<ComponentsDef> = None;
        for (k, v) in map.into_iter() {
            // Skip known/consumed top-level fields so they don't end up
            // repeated. We only extract a typed `components` object and
            // ignore other arbitrary properties.
            if k == "id" || k == "entity_type" || k == "x" || k == "y" || k == "z_index" || k == "layer" || k == "name" {
                continue;
            }
            if k == "components" {
                // `components` may be an object or a serialized string; accept both.
                match v {
                    serde_json::Value::Object(_) => {
                        components = serde_json::from_value::<ComponentsDef>(v).ok();
                    }
                    serde_json::Value::String(s) => {
                        components = serde_json::from_str::<ComponentsDef>(&s).ok();
                    }
                    _ => {
                        // ignore non-object/string components
                    }
                }
            }
        }

        Ok(LevelEntity {
            id,
            entity_type: et,
            x,
            y,
            z_index,
            name,
            layer,
            components,
        })
    }
}

/// Cached representation of a loaded level. The GameView can insert this
/// Resource (or call `refresh`) to make a loaded level available to systems.
#[derive(Resource, Debug, Default, Clone)]
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
        // Load all entity type JSON files from assets/entity_types and register
        // them so `parse_level_definition` (which deserializes LevelEntity)
        // can resolve typed entity types.
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
            let mut et: crate::game::level::types::EntityTypeDefinition = serde_json::from_str(&txt).expect("parse et json");
            et.key = stem.clone();
            et_map.insert(stem, et);
        }

        // Register entity types for deserialization
        crate::game::level::types::register_entity_types(et_map.clone()).expect("register entity types");

        let lvl = parse_level_definition(&content).expect("should parse level json");
        let entities = lvl.entities.expect("entities present");
        assert!(entities.len() > 0, "expected some entities");

        // Ensure every entity's entity_type key matches a loaded type
        for e in &entities {
            assert!(et_map.contains_key(&e.entity_type.key), "missing entity type for {}", e.entity_type.key);
        }
    }

    // (health parsing tests removed)
}


