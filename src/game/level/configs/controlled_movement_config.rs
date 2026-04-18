use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ControlledMovementConfig {
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub jump_force: Option<f32>,
    #[serde(default)] pub allow_double_jump: Option<bool>,
    #[serde(default)] pub jumps_performed: Option<u8>,
    #[serde(default)] pub dash_force: Option<f32>,
    #[serde(default)] pub max_speed: Option<f32>,
    #[serde(default)] pub facing: Option<f32>,
}

