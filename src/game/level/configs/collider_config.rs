use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ColliderConfig {
    #[serde(default)] pub offset: Option<[f32;2]>,
    #[serde(default)] pub is_trigger: Option<bool>,
    #[serde(default)] pub rectangle_half_extents: Option<[f32;2]>,
    #[serde(default)] pub circle_radius: Option<f32>,
}

