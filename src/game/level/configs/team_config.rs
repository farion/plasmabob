use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TeamConfig {
    #[serde(default)] pub name: Option<String>,
}

