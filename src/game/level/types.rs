use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    CollectibleEffectConfig,
};
// StateMachineConfig is parsed as a typed config; we don't store a runtime
// StateMachine component in ComponentsDef anymore.

// ─── Level bounds ─────────────────────────────────────────────────────────────

/// World-space dimensions of a level (width × height in pixels/units).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelBounds {
    pub width: f32,
    pub height: f32,
}

impl Default for LevelBounds {
    fn default() -> Self {
        LevelBounds { width: 4096.0, height: 1024.0 }
    }
}

impl LevelBounds {
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

// ─── State machine types ──────────────────────────────────────────────────────

/// Typed state machine configuration parsed from an entity type JSON.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct StateMachineConfig {
    pub initial_state: String,
    #[serde(default)]
    pub states: HashMap<String, StateConfig>,
}

/// Configuration for a single animation state.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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

fn default_entity_types_path() -> String {
    "entity_types".to_string()
}

pub fn normalize_asset_reference(reference: &str) -> String {
    reference.trim().trim_start_matches("assets/").to_string()
}

// ─── Primitive property values ────────────────────────────────────────────────

/// Minimal runtime representation of a level file used by the Game view.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelDefinition {
    #[serde(default)]
    pub terrain: Option<TerrainDefinition>,
    #[serde(default, deserialize_with = "deserialize_opt_vec_string_or_seq")]
    pub music: Option<Vec<String>>,
    #[serde(default)]
    pub quotes: Vec<String>,
    #[serde(default = "default_entity_types_path")]
    pub entity_types_path: String,
    /// Entities defined in this level.
    #[serde(default)]
    pub entities: Vec<LevelEntity>,
    /// World bounds (width × height in world units).
    #[serde(default)]
    pub bounds: Option<LevelBounds>,
    /// Asset path to the level background image.
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerrainDefinition {
    #[serde(default)]
    pub background: Option<String>,
}

/// Minimal runtime representation of an entity type JSON.
///
/// Note: older formats used a `component: ["Name"]` array. The current
/// format uses a `components: { "ComponentName": { ...attrs... }, ... }`
/// object where keys are component names and values are optional config
/// objects for that component.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityTypeDefinition {
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
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
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

    pub fn state_machine(&self) -> Option<StateMachineConfig> {
        self.state_machine_config()
    }

    pub fn set_state_machine(&mut self, state_machine: StateMachineConfig) -> Result<(), serde_json::Error> {
        let components = self.components.get_or_insert_with(ComponentsDef::default);
        components.state_machine = Some(state_machine);
        Ok(())
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width.unwrap_or_default(), self.height.unwrap_or_default())
    }

    pub fn component_names(&self) -> Vec<String> {
        let Some(comps) = self.components.as_ref() else {
            return Vec::new();
        };

        let mut names = Vec::new();
        if comps.health.is_some() { names.push("health".to_string()); }
        if comps.controlled_movement.is_some() { names.push("controlled_movement".to_string()); }
        if comps.auto_movement.is_some() { names.push("auto_movement".to_string()); }
        if comps.moving_platform.is_some() { names.push("moving_platform".to_string()); }
        if comps.rigid_body.is_some() { names.push("rigid_body".to_string()); }
        if comps.gravity.is_some() { names.push("gravity".to_string()); }
        if comps.blocking.is_some() { names.push("blocking".to_string()); }
        if comps.controlled_range_attack.is_some() { names.push("controlled_range_attack".to_string()); }
        if comps.auto_range_attack.is_some() { names.push("auto_range_attack".to_string()); }
        if comps.auto_melee_attack.is_some() { names.push("auto_melee_attack".to_string()); }
        if comps.controlled_melee_attack.is_some() { names.push("controlled_melee_attack".to_string()); }
        if comps.damageable.is_some() { names.push("damageable".to_string()); }
        if comps.team.is_some() { names.push("team".to_string()); }
        if comps.orientation.is_some() { names.push("orientation".to_string()); }
        if comps.state_machine.is_some() { names.push("state_machine".to_string()); }
        if comps.collider.is_some() { names.push("collider".to_string()); }
        if comps.collectible_effect.is_some() { names.push("collectible_effect".to_string()); }
        names
    }

    pub fn has_component(&self, name: &str) -> bool {
        self.component_names().iter().any(|n| n == name)
    }

    pub fn set_component_names(&mut self, names: &[String]) {
        let mut wanted = std::collections::HashSet::<String>::new();
        for name in names {
            wanted.insert(name.to_ascii_lowercase());
        }

        let comps = self.components.get_or_insert_with(ComponentsDef::default);
        comps.health = wanted.contains("health").then(HealthConfig::default);
        comps.controlled_movement = wanted.contains("controlled_movement").then(ControlledMovementConfig::default);
        comps.auto_movement = wanted.contains("auto_movement").then(AutoMovementConfig::default);
        comps.moving_platform = wanted.contains("moving_platform").then(MovingPlatformConfig::default);
        comps.rigid_body = wanted.contains("rigid_body").then(RigidBodyConfig::default);
        comps.gravity = wanted.contains("gravity").then(GravityConfig::default);
        comps.blocking = wanted.contains("blocking").then(BlockingConfig::default);
        comps.controlled_range_attack = wanted.contains("controlled_range_attack").then(ControlledRangeAttackConfig::default);
        comps.auto_range_attack = wanted.contains("auto_range_attack").then(AutoRangeAttackConfig::default);
        comps.auto_melee_attack = wanted.contains("auto_melee_attack").then(AutoMeleeAttackConfig::default);
        comps.controlled_melee_attack = wanted.contains("controlled_melee_attack").then(ControlledMeleeAttackConfig::default);
        comps.damageable = wanted.contains("damageable").then(DamageableConfig::default);
        comps.team = wanted.contains("team").then(TeamConfig::default);
        comps.orientation = wanted.contains("orientation").then(OrientationConfig::default);
        comps.state_machine = wanted.contains("state_machine").then(StateMachineConfig::default);
        comps.collider = wanted.contains("collider").then(ColliderConfig::default);
        comps.collectible_effect = wanted.contains("collectible_effect").then(|| CollectibleEffectConfig { heal: None });
    }

    pub fn component_attribute_value(&self, component: &str, attribute: &str) -> Option<serde_json::Value> {
        let comps = self.components.as_ref()?;
        let value = match component.to_ascii_lowercase().as_str() {
            "health" => serde_json::to_value(comps.health.as_ref()?).ok()?,
            "controlled_movement" => serde_json::to_value(comps.controlled_movement.as_ref()?).ok()?,
            "auto_movement" => serde_json::to_value(comps.auto_movement.as_ref()?).ok()?,
            "moving_platform" => serde_json::to_value(comps.moving_platform.as_ref()?).ok()?,
            "rigid_body" => serde_json::to_value(comps.rigid_body.as_ref()?).ok()?,
            "gravity" => serde_json::to_value(comps.gravity.as_ref()?).ok()?,
            "blocking" => serde_json::to_value(comps.blocking.as_ref()?).ok()?,
            "controlled_range_attack" => serde_json::to_value(comps.controlled_range_attack.as_ref()?).ok()?,
            "auto_range_attack" => serde_json::to_value(comps.auto_range_attack.as_ref()?).ok()?,
            "auto_melee_attack" => serde_json::to_value(comps.auto_melee_attack.as_ref()?).ok()?,
            "controlled_melee_attack" => serde_json::to_value(comps.controlled_melee_attack.as_ref()?).ok()?,
            "damageable" => serde_json::to_value(comps.damageable.as_ref()?).ok()?,
            "team" => serde_json::to_value(comps.team.as_ref()?).ok()?,
            "orientation" => serde_json::to_value(comps.orientation.as_ref()?).ok()?,
            "state_machine" => serde_json::to_value(comps.state_machine.as_ref()?).ok()?,
            "collider" => serde_json::to_value(comps.collider.as_ref()?).ok()?,
            "collectible_effect" => serde_json::to_value(comps.collectible_effect.as_ref()?).ok()?,
            _ => return None,
        };
        // Treat explicit JSON null the same as "absent": return None so
        // the editor does not consider a null-valued attribute an explicit
        // override. This prevents the UI from showing a Clear button for
        // fields that were not actually set in the source JSON (they would
        // otherwise be serialized as `null` when converting typed configs
        // to a serde_json::Value).
        let obj = value.as_object()?;
        let v = obj.get(attribute).cloned()?;
        if v.is_null() {
            None
        } else {
            Some(v)
        }
    }

    pub fn set_component_attribute_value(&mut self, component: &str, attribute: &str, value: serde_json::Value) {
        let mut root = self
            .components
            .as_ref()
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();

        let entry = root
            .entry(component.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        let object = if let Some(obj) = entry.as_object_mut() {
            obj
        } else {
            *entry = serde_json::Value::Object(serde_json::Map::new());
            entry.as_object_mut().expect("components entry must be object")
        };
        // Coerce float JSON numbers without fractional part into integers
        // to avoid deserialization failures for typed integer fields
        // (e.g. HealthConfig.current: Option<u32>) when the UI provides
        // 10.0 instead of 10.
        let final_value = match value {
            serde_json::Value::Number(n) => {
                if n.as_i64().is_none() {
                    if let Some(f) = n.as_f64() {
                        if (f.fract()).abs() < std::f64::EPSILON {
                            // Safe to coerce to i64 if within i64 range
                            let int_val = f as i64;
                            serde_json::Value::Number(serde_json::Number::from(int_val))
                        } else {
                            serde_json::Value::Number(n)
                        }
                    } else {
                        serde_json::Value::Number(n)
                    }
                } else {
                    serde_json::Value::Number(n)
                }
            }
            other => other,
        };

        object.insert(attribute.to_string(), final_value);

        // Attempt to convert the updated raw JSON `root` back into a
        // typed `ComponentsDef`. If deserialization fails (for example
        // because a floating JSON number cannot be coerced into an
        // integer field like `u32`), do not overwrite `self.components`.
        // Overwriting with `None` would remove the component entirely
        // and make the UI drop the component table unexpectedly.
        match serde_json::from_value::<ComponentsDef>(serde_json::Value::Object(root)) {
            Ok(parts) => {
                self.components = Some(parts);
            }
            Err(err) => {
                tracing::warn!(component = component, attribute = attribute, error = %err, "Failed to apply component attribute update: preserving previous components to avoid data loss");
            }
        }
    }

    pub fn remove_component_attribute(&mut self, component: &str, attribute: &str) {
        let Some(mut root) = self
            .components
            .as_ref()
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| v.as_object().cloned())
        else {
            return;
        };

        if let Some(component_value) = root.get_mut(component) {
            if let Some(component_object) = component_value.as_object_mut() {
                component_object.remove(attribute);
            }
        }

        self.components = if root.is_empty() {
            None
        } else {
            serde_json::from_value(serde_json::Value::Object(root)).ok()
        };
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

/// Typed representation of the `components` object in an entity type JSON.
///
/// This structure holds actual runtime component instances (constructed
/// from defaults and then overridden with JSON when present). We implement
/// a custom `Deserialize` to accept the JSON `components` object and build
/// component instances by applying `override_from_json`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ComponentsDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controlled_movement: Option<ControlledMovementConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_movement: Option<AutoMovementConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_platform: Option<MovingPlatformConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rigid_body: Option<RigidBodyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gravity: Option<GravityConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<BlockingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controlled_range_attack: Option<ControlledRangeAttackConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_range_attack: Option<AutoRangeAttackConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_melee_attack: Option<AutoMeleeAttackConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controlled_melee_attack: Option<ControlledMeleeAttackConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damageable: Option<DamageableConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<OrientationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_machine: Option<StateMachineConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collider: Option<ColliderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collectible_effect: Option<CollectibleEffectConfig>,
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
                "collectible_effect" | "collectibleeffect" => out.collectible_effect = Some(serde_json::from_value::<CollectibleEffectConfig>(val).map_err(serde::de::Error::custom)?),
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
pub struct LevelEntity {
    pub id: String,
    pub entity_type: String,
    pub x: f32,
    pub y: f32,
    pub z_index: Option<f32>,
    pub name: Option<String>,
    pub layer: Option<String>,
    /// Optional per-level components overrides parsed from the level JSON's
    /// `components` object. This is a typed `ComponentsDef` (mirrors
    /// `EntityTypeDefinition.components`).
    pub components: Option<ComponentsDef>,
    pub extra: HashMap<String, serde_json::Value>,
}

impl Serialize for LevelEntity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();
        map.insert("id".to_string(), serde_json::Value::String(self.id.clone()));
        map.insert("entity_type".to_string(), serde_json::Value::String(self.entity_type.clone()));
        map.insert("x".to_string(), serde_json::json!(self.x));
        map.insert("y".to_string(), serde_json::json!(self.y));
        if let Some(z_index) = self.z_index {
            map.insert("z_index".to_string(), serde_json::json!(z_index));
        }
        if let Some(name) = &self.name {
            map.insert("name".to_string(), serde_json::Value::String(name.clone()));
        }
        if let Some(layer) = &self.layer {
            map.insert("layer".to_string(), serde_json::Value::String(layer.clone()));
        }
        if let Some(components) = &self.components {
            let value = serde_json::to_value(components).map_err(serde::ser::Error::custom)?;
            map.insert("components".to_string(), value);
        }
        for (k, v) in &self.extra {
            map.insert(k.clone(), v.clone());
        }
        serde_json::Value::Object(map).serialize(serializer)
    }
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

        let x = map.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = map.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let z_index = map.get("z_index").and_then(|v| v.as_f64()).map(|v| v as f32);
        let layer = map.get("layer").and_then(|v| v.as_str()).map(|s| s.to_string());
        let name = map.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut components: Option<ComponentsDef> = None;
        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
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
                continue;
            }
            extra.insert(k, v);
        }

        Ok(LevelEntity {
            id,
            entity_type: et_key,
            x,
            y,
            z_index,
            name,
            layer,
            components,
            extra,
        })
    }
}

impl LevelEntity {
    pub fn set_component_attribute_value(&mut self, component: &str, attribute: &str, value: serde_json::Value) {
        let mut root = self
            .components
            .as_ref()
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();

        let entry = root
            .entry(component.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        let object = if let Some(obj) = entry.as_object_mut() {
            obj
        } else {
            *entry = serde_json::Value::Object(serde_json::Map::new());
            entry.as_object_mut().expect("components entry must be object")
        };
        // Coerce float JSON numbers without fractional part into integers
        // to avoid deserialization failures for typed integer fields
        // (e.g. HealthConfig.current: Option<u32>) when the UI provides
        // 10.0 instead of 10.
        let final_value = match value {
            serde_json::Value::Number(n) => {
                if n.as_i64().is_none() {
                    if let Some(f) = n.as_f64() {
                        if (f.fract()).abs() < std::f64::EPSILON {
                            let int_val = f as i64;
                            serde_json::Value::Number(serde_json::Number::from(int_val))
                        } else {
                            serde_json::Value::Number(n)
                        }
                    } else {
                        serde_json::Value::Number(n)
                    }
                } else {
                    serde_json::Value::Number(n)
                }
            }
            other => other,
        };

        object.insert(attribute.to_string(), final_value);

        // See comment in EntityTypeDefinition::set_component_attribute_value
        // — avoid clobbering `self.components` with `None` when
        // deserialization fails (e.g. type mismatch). Log a warning so
        // we can investigate problematic edits.
        match serde_json::from_value::<ComponentsDef>(serde_json::Value::Object(root)) {
            Ok(parts) => {
                self.components = Some(parts);
            }
            Err(err) => {
                tracing::warn!(component = component, attribute = attribute, error = %err, "Failed to apply component attribute update to level entity: preserving previous components");
            }
        }
    }

    pub fn remove_component_attribute(&mut self, component: &str, attribute: &str) {
        let Some(mut root) = self
            .components
            .as_ref()
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| v.as_object().cloned())
        else {
            return;
        };

        if let Some(component_value) = root.get_mut(component) {
            if let Some(component_object) = component_value.as_object_mut() {
                component_object.remove(attribute);
                if component_object.is_empty() {
                    root.remove(component);
                }
            }
        }

        self.components = if root.is_empty() {
            None
        } else {
            serde_json::from_value(serde_json::Value::Object(root)).ok()
        };
    }
}

/// Cached representation of a loaded level. The GameView can insert this
/// Resource (or call `refresh`) to make a loaded level available to systems.
#[derive(Resource, Debug, Default, Clone)]
pub struct CachedLevelDefinition {
    pub level: Option<LevelDefinition>,
    pub entity_types: HashMap<String, EntityTypeDefinition>,
}

impl StateConfig {
    pub fn hitbox_points(&self) -> &[[f32; 2]] {
        self.collider_box.as_deref().unwrap_or(&[])
    }
}

// --- Parsing helpers for unit tests ---
// Keep the helper available only for tests to avoid unused-code warnings
#[cfg(test)]
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
        let json = r#"{"entities":[]}"#;
        let lvl = parse_level_definition(json).expect("should parse");
        assert!(lvl.entities.is_empty());
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

        let lvl = parse_level_definition(&content).expect("should parse level json");
        let entities = lvl.entities;
        assert!(entities.len() > 0, "expected some entities");

        // Ensure every entity's entity_type key matches a loaded type
        for e in &entities {
            assert!(et_map.contains_key(&e.entity_type), "missing entity type for {}", e.entity_type);
        }
    }

    // (health parsing tests removed)
}


