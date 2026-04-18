use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AutoMovementConfig {
    #[serde(default)] pub direction: Option<[f32; 2]>,
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub enabled: Option<bool>,
}

