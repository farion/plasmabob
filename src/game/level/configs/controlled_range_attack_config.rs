use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ControlledRangeAttackConfig {
    #[serde(default)] pub damage: Option<i32>,
    #[serde(default)] pub range: Option<f32>,
    #[serde(default)] pub speed: Option<f32>,
    /// Cooldown in milliseconds
    #[serde(default)] pub cooldown: Option<u64>,
    #[serde(default)] pub projectile_type: Option<String>,
    #[serde(default)] pub shoot_effect: Option<String>,
    #[serde(default)] pub impact_effect: Option<String>,
}


