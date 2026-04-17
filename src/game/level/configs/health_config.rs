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
