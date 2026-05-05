use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ControlledMeleeAttackConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f32>,
    /// Cooldown in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<u64>,
}
