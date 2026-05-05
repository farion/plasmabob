use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ControlledMovementConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_force: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_double_jump: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jumps_performed: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash_force: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facing: Option<f32>,
}
