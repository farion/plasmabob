use serde::Deserialize;

/// Configuration for collectible effects parsed from entity-type JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct CollectibleEffectConfig {
    /// Optional heal amount (unsigned in authored JSON).
    #[serde(default)]
    pub heal: Option<u32>,
}

