use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TeamConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
