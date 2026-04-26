use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AutoRangeAttackConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggro_range: Option<f32>,
    /// Cooldown in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_effect: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shoot_effect: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_effect: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
