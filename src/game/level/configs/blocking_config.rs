use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BlockingConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks_line_of_sight: Option<bool>,
}

