use serde::{Deserialize, Serialize};

/// Configuration for collectible effects parsed from entity-type JSON.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CollectibleEffectConfig {
    /// Optional heal amount (unsigned in authored JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heal: Option<u32>,
}
