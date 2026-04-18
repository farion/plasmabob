use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DamageableConfig {
    #[serde(default)] pub damaged_duration_secs: Option<f32>,
}

