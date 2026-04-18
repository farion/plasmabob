use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AutoRangeAttackConfig {
    #[serde(default)] pub damage: Option<i32>,
    #[serde(default)] pub range: Option<f32>,
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub aggro_range: Option<f32>,
    /// Cooldown in milliseconds
    #[serde(default)] pub cooldown: Option<u64>,
    #[serde(default)] pub particle_effect: Option<String>,
    #[serde(default)] pub shoot_effect: Option<String>,
    #[serde(default)] pub impact_effect: Option<String>,
    #[serde(default)] pub enabled: Option<bool>,
}


