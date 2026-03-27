use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

const DEFAULT_ANIMATION_FRAME_MS: u64 = 500;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LevelDefinition {
    pub(crate) terrain: TerrainDefinition,
    pub(crate) music: String,
    #[serde(default)]
    pub(crate) quotes: Vec<String>,
    #[serde(default)]
    pub(crate) bounds: Option<LevelBoundsDefinition>,
    pub(crate) entity_types: HashMap<String, EntityTypeDefinition>,
    pub(crate) entities: Vec<EntityDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LevelBoundsDefinition {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TerrainDefinition {
    pub(crate) background: String,
}

impl LevelDefinition {
    pub(crate) fn terrain_background_asset_path(&self) -> String {
        normalize_asset_reference(&self.terrain.background)
    }

    pub(crate) fn music_asset_path(&self) -> String {
        normalize_asset_reference(&self.music)
    }

    pub(crate) fn quote_asset_paths(&self) -> Vec<String> {
        self.quotes
            .iter()
            .map(|quote| normalize_asset_reference(quote))
            .collect()
    }

    pub(crate) fn bounds_size(&self) -> Option<Vec2> {
        self.bounds.as_ref().map(LevelBoundsDefinition::size)
    }
}

impl LevelBoundsDefinition {
    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EntityTypeDefinition {
    #[serde(rename = "component")]
    pub(crate) components: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) disposition: Option<String>,
    #[serde(default)]
    pub(crate) animations: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) hitbox: Vec<[f32; 2]>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    #[serde(default)]
    pub(crate) health: Option<i32>,
    #[serde(default)]
    pub(crate) damage: Option<i32>,
    /// Default z-index for all instances of this type. Can be overridden per entity.
    #[serde(default)]
    pub(crate) z_index: Option<f32>,
    /// Maximum range of the plasma beam (player only). Enables the PlasmaAttack component.
    #[serde(default)]
    pub(crate) attack_range: Option<f32>,
    /// Optional per-entity-type frame interval for cycling animation arrays.
    #[serde(default)]
    pub(crate) animation_frame_ms: Option<u64>,
}

impl EntityTypeDefinition {
    pub(crate) fn normalized_animations(&self) -> HashMap<String, Vec<String>> {
        self.animations
            .iter()
            .map(|(name, frames)| {
                (
                    name.clone(),
                    frames
                        .iter()
                        .map(|frame| normalize_asset_reference(frame))
                        .collect(),
                )
            })
            .collect()
    }

    pub(crate) fn default_animation_path(&self) -> Option<String> {
        self.animations
            .get("default")
            .and_then(|frames| frames.first())
            .map(|path| normalize_asset_reference(path))
    }

    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub(crate) fn centered_hitbox_polygon(&self) -> Result<Vec<Vec2>, String> {
        let points: Vec<[f32; 2]> = if self.hitbox.is_empty() {
            vec![
                [0.0, 0.0],
                [self.width, 0.0],
                [self.width, self.height],
                [0.0, self.height],
            ]
        } else {
            self.hitbox.clone()
        };

        if points.len() < 3 {
            return Err("hitbox polygon requires at least 3 points".to_string());
        }

        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;

        Ok(points
            .iter()
            .map(|point| Vec2::new(point[0] - half_width, point[1] - half_height))
            .collect())
    }

    pub(crate) fn animation_frame_seconds(&self) -> f32 {
        let interval_ms = self.animation_frame_ms.unwrap_or(DEFAULT_ANIMATION_FRAME_MS);
        (interval_ms as f32 / 1000.0).max(0.001)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EntityDefinition {
    pub(crate) id: String,
    pub(crate) entity_type: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    /// Per-instance z-index override. Takes precedence over the entity type's z_index.
    #[serde(default)]
    pub(crate) z_index: Option<f32>,
}

#[derive(Debug)]
pub(crate) enum LoadLevelError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl Display for LoadLevelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
        }
    }
}

impl Error for LoadLevelError {}

impl From<std::io::Error> for LoadLevelError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for LoadLevelError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}

pub(crate) fn load_level_from_asset_path(asset_path: &str) -> Result<LevelDefinition, LoadLevelError> {
    let content = std::fs::read_to_string(asset_path_to_filesystem_path(asset_path))?;
    Ok(serde_json::from_str(&content)?)
}

pub(crate) fn asset_path_to_filesystem_path(asset_path: &str) -> PathBuf {
    Path::new("assets").join(normalize_asset_reference(asset_path))
}

pub(crate) fn normalize_asset_reference(reference: &str) -> String {
    reference.trim().trim_start_matches("assets/").to_string()
}

pub(crate) fn bottom_left_to_world(
    window_size: Vec2,
    x: f32,
    y: f32,
    entity_size: Vec2,
    z: f32,
) -> Vec3 {
    Vec3::new(
        x - (window_size.x * 0.5) + (entity_size.x * 0.5),
        y - (window_size.y * 0.5) + (entity_size.y * 0.5),
        z,
    )
}

pub(crate) fn clamp_level_position(x: f32, y: f32, entity_size: Vec2, level_size: Vec2) -> Vec2 {
    let max_x = (level_size.x - entity_size.x).max(0.0);
    let max_y = (level_size.y - entity_size.y).max(0.0);

    Vec2::new(x.clamp(0.0, max_x), y.clamp(0.0, max_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_level_schema() {
        let json = r#"
        {
            "terrain": {
                "background": "assets/backgrounds/level1.png"
            },
            "music": "assets/music/level1.ogg",
            "bounds": {
                "width": 1584,
                "height": 1024
            },
            "entity_types": {
                "dirt": {
                    "component": ["floor"],
                    "animations": {
                        "default": ["assets/dirt/default1.png", "assets/dirt/default2.png"]
                    },
                    "width": 100,
                    "height": 20
                },
                "cockroach": {
                    "component": ["npc", "hostile"],
                    "disposition": "hostile",
                    "animations": {
                        "default": ["assets/cockroach/default1.png", "assets/cockroach/default2.png"],
                        "walk": [],
                        "jump": [],
                        "die": [],
                        "hit": [],
                        "fight": []
                    },
                    "width": 100,
                    "height": 20
                },
                "bob": {
                    "component": ["player"],
                    "animation_frame_ms": 250,
                    "animations": {
                        "default": ["assets/bob/default1.png", "assets/bob/default2.png"],
                        "walk": [],
                        "jump": [],
                        "die": [],
                        "hit": [],
                        "fight": []
                    },
                    "width": 100,
                    "height": 20
                }
            },
            "entities": [
                {
                    "id": "dirt1",
                    "entity_type": "dirt",
                    "x": 10,
                    "y": 20
                },
                {
                    "id": "player",
                    "entity_type": "bob",
                    "x": 10,
                    "y": 20
                }
            ]
        }
        "#;

        let parsed: LevelDefinition = serde_json::from_str(json).expect("schema should parse");

        assert_eq!(parsed.entity_types.len(), 3);
        assert_eq!(parsed.entities.len(), 2);
        assert_eq!(parsed.bounds_size(), Some(Vec2::new(1584.0, 1024.0)));
        assert_eq!(parsed.terrain_background_asset_path(), "backgrounds/level1.png");
        assert_eq!(parsed.music_asset_path(), "music/level1.ogg");
        assert!(parsed.quote_asset_paths().is_empty());
        assert_eq!(parsed.entity_types["dirt"].components, vec!["floor"]);
        assert_eq!(parsed.entity_types["cockroach"].disposition.as_deref(), Some("hostile"));
        assert_eq!(parsed.entity_types["bob"].width, 100.0);
        assert_eq!(parsed.entity_types["bob"].animation_frame_ms, Some(250));
    }

    #[test]
    fn uses_default_animation_frame_interval_when_missing() {
        let json = r#"
        {
            "terrain": { "background": "assets/backgrounds/level1.png" },
            "music": "assets/music/level1.ogg",
            "entity_types": {
                "dummy": {
                    "component": ["npc"],
                    "animations": { "default": ["assets/dirt/default1.png"] },
                    "width": 16,
                    "height": 16
                }
            },
            "entities": [
                { "id": "dummy1", "entity_type": "dummy", "x": 0, "y": 0 }
            ]
        }
        "#;

        let parsed: LevelDefinition = serde_json::from_str(json).expect("schema should parse");
        assert_eq!(parsed.entity_types["dummy"].animation_frame_seconds(), 0.5);
    }

    #[test]
    fn strips_assets_prefix_from_asset_references() {
        assert_eq!(normalize_asset_reference("assets/levels/level1.json"), "levels/level1.json");
        assert_eq!(normalize_asset_reference("bob/default1.png"), "bob/default1.png");
    }

    #[test]
    fn converts_bottom_left_level_coordinates_to_world_space() {
        let world = bottom_left_to_world(
            Vec2::new(800.0, 600.0),
            10.0,
            20.0,
            Vec2::new(100.0, 20.0),
            0.0,
        );

        assert_eq!(world, Vec3::new(-340.0, -270.0, 0.0));
    }

    #[test]
    fn clamps_level_positions_to_valid_bounds() {
        let position = clamp_level_position(
            1700.0,
            -30.0,
            Vec2::new(98.0, 128.0),
            Vec2::new(1584.0, 1024.0),
        );

        assert_eq!(position, Vec2::new(1486.0, 0.0));
    }
}



