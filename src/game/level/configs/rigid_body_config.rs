use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RigidBodyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear_damp: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restitution: Option<f32>,
}
