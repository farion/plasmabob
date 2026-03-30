use std::collections::HashMap;

use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

const DEFAULT_ENTITY_TYPES_PATH: &str = "entity_types";

fn default_entity_types_path() -> String {
    DEFAULT_ENTITY_TYPES_PATH.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelFile {
    pub(crate) terrain: TerrainDefinition,
    #[serde(default)]
    pub(crate) quotes: Vec<String>,
    pub(crate) music: String,
    #[serde(default)]
    pub(crate) bounds: Option<LevelBoundsDefinition>,
    #[serde(default = "default_entity_types_path")]
    pub(crate) entity_types_path: String,
    pub(crate) entities: Vec<EntityDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TerrainDefinition {
    pub(crate) background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelBoundsDefinition {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl LevelBoundsDefinition {
    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntityTypeStateDefinition {
    #[serde(default)]
    pub(crate) animation: Vec<String>,
    #[serde(default)]
    pub(crate) hitbox: Vec<[f32; 2]>,
    #[serde(default)]
    pub(crate) animation_frame_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntityTypeDefinition {
    #[serde(rename = "component")]
    pub(crate) components: Vec<String>,
    #[serde(default)]
    pub(crate) disposition: Option<String>,
    #[serde(default)]
    pub(crate) states: HashMap<String, EntityTypeStateDefinition>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    #[serde(default)]
    pub(crate) health: Option<i32>,
    #[serde(default)]
    pub(crate) damage: Option<i32>,
    #[serde(default)]
    pub(crate) attack_range: Option<f32>,
}

impl EntityTypeDefinition {
    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub(crate) fn default_texture_asset_path(&self) -> Option<String> {
        self
            .states
            .get("default")
            .and_then(|state| state.animation.first())
            .or_else(|| {
                self.states
                    .values()
                    .flat_map(|state| state.animation.iter())
                    .next()
            })
            .map(|path| normalize_asset_reference(path))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntityDefinition {
    pub(crate) id: String,
    pub(crate) entity_type: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    #[serde(default)]
    pub(crate) z_index: Option<f32>,
}

pub(crate) fn normalize_asset_reference(reference: &str) -> String {
    reference.trim().trim_start_matches("assets/").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_assets_prefix() {
        assert_eq!(normalize_asset_reference("assets/levels/level1.json"), "levels/level1.json");
        assert_eq!(normalize_asset_reference("entity_types/bob.json"), "entity_types/bob.json");
    }

    #[test]
    fn uses_default_animation_as_primary_preview_texture() {
        let entity_type = EntityTypeDefinition {
            components: vec!["player".to_string()],
            disposition: None,
            states: HashMap::from([(
                "default".to_string(),
                EntityTypeStateDefinition {
                    animation: vec!["assets/bob/bob-default.png".to_string()],
                    hitbox: vec![],
                    animation_frame_ms: None,
                },
            )]),
            width: 10.0,
            height: 20.0,
            health: None,
            damage: None,
            attack_range: None,
        };

        assert_eq!(
            entity_type.default_texture_asset_path().as_deref(),
            Some("bob/bob-default.png")
        );
    }
}

