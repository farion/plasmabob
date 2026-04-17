use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RigidBodyConfig {
    #[serde(default)] pub velocity: Option<[f32;2]>,
    #[serde(default)] pub mass: Option<f32>,
    #[serde(default)] pub linear_damp: Option<f32>,
    #[serde(default)] pub restitution: Option<f32>,
}

