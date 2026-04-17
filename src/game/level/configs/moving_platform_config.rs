use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MovingPlatformConfig {
    #[serde(default)] pub waypoints: Option<Vec<[f32;2]>>,
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub repeat: Option<bool>,
    #[serde(default)] pub enabled: Option<bool>,
}

