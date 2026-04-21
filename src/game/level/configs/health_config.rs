use serde::{Deserialize, Serialize};

/// Config used when parsing `components.health` in entity-type or level JSON.
/// All fields are optional so that only present fields override defaults.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HealthConfig {
    /// Maximum HP (if present, also sets current HP when applied at type-level).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub despawn_on_death: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub despawn_delay_ms: Option<u64>,
}
