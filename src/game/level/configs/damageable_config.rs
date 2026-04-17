use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DamageableConfig {
    #[serde(default)] pub damaged_duration_secs: Option<f32>,
}

