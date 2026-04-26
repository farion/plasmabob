use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AutoMovementConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggro: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggro_range: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deaggro_range: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggro_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patrol_range: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patrol_pause_time: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patrol_waypoints: Option<Vec<[f32; 2]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_of_sight: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_angle: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_check_interval: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_engage_distance: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kiting_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kiting_hp_threshold: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_fall_when_following: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_stop_distance: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_on_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_on_aggro: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_on_return_to_origin: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_force: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_cooldown: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceleration: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_timeout: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_aggro_with_team: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggro_sharing_radius: Option<f32>,
}
