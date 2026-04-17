use serde::Deserialize;

/// Config used when parsing `components.health` in entity-type or level JSON.
/// All fields are optional so that only present fields override defaults.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HealthConfig {
    /// Maximum HP (if present, also sets current HP when applied at type-level).
    #[serde(default)]
    pub max: Option<u32>,
    #[serde(default)]
    pub current: Option<u32>,
    #[serde(default)]
    pub despawn_on_death: Option<bool>,
    #[serde(default)]
    pub despawn_delay_ms: Option<u64>,
}

impl HealthConfig {
    /// Merge values from `other` into `self` by replacing any `Some` fields.
    pub fn merge_from(&mut self, other: &HealthConfig) {
        if other.max.is_some() { self.max = other.max; }
        if other.current.is_some() { self.current = other.current; }
        if other.despawn_on_death.is_some() { self.despawn_on_death = other.despawn_on_death; }
        if other.despawn_delay_ms.is_some() { self.despawn_delay_ms = other.despawn_delay_ms; }
    }
}

