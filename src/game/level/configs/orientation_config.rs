use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OrientationConfig {
    #[serde(default)] pub facing: Option<String>,
    #[serde(default)] pub surface_alignment: Option<[f32;2]>,
}

