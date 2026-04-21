use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GravityConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grounded: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_accel: Option<[f32; 2]>,
}
