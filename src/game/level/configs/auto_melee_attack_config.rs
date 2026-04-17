use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AutoMeleeAttackConfig {
    #[serde(default)] pub damage: Option<i32>,
    #[serde(default)] pub range: Option<f32>,
    /// Cooldown in milliseconds
    #[serde(default)] pub cooldown: Option<u64>,
    #[serde(default)] pub enabled: Option<bool>,
}


