use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TeamConfig {
    #[serde(default)] pub name: Option<String>,
}

