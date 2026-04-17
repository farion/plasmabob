use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AutoMovementConfig {
    #[serde(default)] pub direction: Option<[f32; 2]>,
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub enabled: Option<bool>,
}

