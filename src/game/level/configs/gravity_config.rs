use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GravityConfig {
    #[serde(default)] pub scale: Option<f32>,
    #[serde(default)] pub grounded: Option<bool>,
    #[serde(default)] pub extra_accel: Option<[f32;2]>,
}

