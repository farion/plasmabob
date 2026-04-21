use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DamageableConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damaged_duration_secs: Option<f32>,
}
