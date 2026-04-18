use serde::{Deserialize, Serialize};

/// Configuration for collectible effects parsed from entity-type JSON.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectibleEffectConfig {
    /// Optional heal amount (unsigned in authored JSON).
    #[serde(default)]
    pub heal: Option<u32>,
}

