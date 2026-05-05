use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OrientationConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_alignment: Option<[f32; 2]>,
}
