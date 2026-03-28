use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

const DEFAULT_ANIMATION_FRAME_MS: u64 = 500;
const DEFAULT_ENTITY_TYPES_PATH: &str = "entity_types.json";

#[derive(Debug, Clone)]
pub(crate) struct LevelDefinition {
    pub(crate) terrain: TerrainDefinition,
    pub(crate) music: String,
    pub(crate) quotes: Vec<String>,
    pub(crate) bounds: Option<LevelBoundsDefinition>,
    pub(crate) entity_types: HashMap<String, EntityTypeDefinition>,
    pub(crate) entities: Vec<EntityDefinition>,
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct CachedLevelDefinition {
    asset_path: String,
    loaded_level: Result<LevelDefinition, String>,
}

impl CachedLevelDefinition {
    pub(crate) fn preload(asset_path: &str) -> Self {
        let normalized_asset_path = normalize_asset_reference(asset_path);
        let loaded_level = load_level_from_asset_path(&normalized_asset_path)
            .map_err(|error| error.to_string());

        Self {
            asset_path: normalized_asset_path,
            loaded_level,
        }
    }

    pub(crate) fn asset_path(&self) -> &str {
        &self.asset_path
    }

    pub(crate) fn level_definition(&self) -> Result<&LevelDefinition, &str> {
        self.loaded_level
            .as_ref()
            .map_err(|error| error.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawLevelDefinition {
    terrain: TerrainDefinition,
    music: String,
    #[serde(default)]
    quotes: Vec<String>,
    #[serde(default)]
    bounds: Option<LevelBoundsDefinition>,
    #[serde(default = "default_entity_types_path")]
    entity_types_path: String,
    entities: Vec<EntityDefinition>,
}

fn default_entity_types_path() -> String {
    DEFAULT_ENTITY_TYPES_PATH.to_string()
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
    /// Per-instance z-index for draw order. When omitted, setup code falls back to component heuristics.
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
    let raw_level: RawLevelDefinition = serde_json::from_str(&content)?;

    let entity_types_asset_path = resolve_entity_types_asset_path(asset_path, &raw_level.entity_types_path);
    let entity_types_content = std::fs::read_to_string(asset_path_to_filesystem_path(&entity_types_asset_path))?;
    let entity_types: HashMap<String, EntityTypeDefinition> = serde_json::from_str(&entity_types_content)?;

    Ok(LevelDefinition {
        terrain: raw_level.terrain,
        music: raw_level.music,
        quotes: raw_level.quotes,
        bounds: raw_level.bounds,
        entity_types,
        entities: raw_level.entities,
    })
}

fn resolve_entity_types_asset_path(level_asset_path: &str, configured_path: &str) -> String {
    let normalized_level_path = normalize_asset_reference(level_asset_path);
    let normalized_configured_path = normalize_asset_reference(configured_path);

    if normalized_configured_path.contains('/') {
        return normalized_configured_path;
    }

    let level_directory = Path::new(&normalized_level_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    let resolved = level_directory.join(normalized_configured_path);
    resolved.to_string_lossy().to_string()
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("plasmabob-tests-{unique}"))
    }

    fn write_temp_file(root: &Path, relative_path: &str, content: &str) -> String {
        let full_path = root.join(relative_path);

        std::fs::create_dir_all(
            full_path
                .parent()
                .expect("temporary file path should have a parent directory"),
        )
        .expect("temporary directory should be created");

        std::fs::write(&full_path, content).expect("temporary file should be written");
        full_path.to_string_lossy().to_string()
    }

    fn in_temp_working_directory<F: FnOnce()>(test: F) {
        struct WorkingDirGuard {
            previous: PathBuf,
        }

        impl Drop for WorkingDirGuard {
            fn drop(&mut self) {
                std::env::set_current_dir(&self.previous)
                    .expect("should restore previous working directory");
            }
        }

        let previous = std::env::current_dir().expect("current directory should be readable");
        let _guard = WorkingDirGuard { previous };
        let root = unique_temp_root();
        std::fs::create_dir_all(root.join("assets")).expect("temporary assets directory should exist");
        std::env::set_current_dir(&root).expect("should switch to temporary working directory");
        test();
    }

    #[test]
    fn parses_the_split_level_schema() {
        in_temp_working_directory(|| {
            let root = std::env::current_dir().expect("current directory should be readable");
            let level_path = write_temp_file(
                &root,
                "assets/levels/level.json",
                r#"
                {
                    "terrain": {
                        "background": "assets/backgrounds/level1.png"
                    },
                    "music": "assets/music/level1.ogg",
                    "bounds": {
                        "width": 1584,
                        "height": 1024
                    },
                    "entity_types_path": "entity_types.json",
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
                            "y": 20,
                            "z_index": 20
                        }
                    ]
                }
                "#,
            );

            write_temp_file(
                &root,
                "assets/levels/entity_types.json",
                r#"
                {
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
                }
                "#,
            );

            let parsed = load_level_from_asset_path(&level_path).expect("schema should parse");

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
            assert_eq!(parsed.entities[1].z_index, Some(20.0));
        });
    }

    #[test]
    fn uses_default_entity_types_file_when_field_is_missing() {
        in_temp_working_directory(|| {
            let root = std::env::current_dir().expect("current directory should be readable");
            let level_path = write_temp_file(
                &root,
                "assets/levels/level.json",
                r#"
                {
                    "terrain": { "background": "assets/backgrounds/level1.png" },
                    "music": "assets/music/level1.ogg",
                    "entities": [
                        { "id": "dummy1", "entity_type": "dummy", "x": 0, "y": 0 }
                    ]
                }
                "#,
            );

            write_temp_file(
                &root,
                "assets/levels/entity_types.json",
                r#"
                {
                    "dummy": {
                        "component": ["npc"],
                        "animations": { "default": ["assets/dirt/default1.png"] },
                        "width": 16,
                        "height": 16
                    }
                }
                "#,
            );

            let parsed = load_level_from_asset_path(&level_path).expect("schema should parse");
            assert_eq!(parsed.entity_types["dummy"].animation_frame_seconds(), 0.5);
        });
    }

    #[test]
    fn resolves_relative_entity_types_path_against_level_directory() {
        assert_eq!(
            resolve_entity_types_asset_path("levels/planet1/level.json", "entity_types.json"),
            "levels/planet1/entity_types.json"
        );
        assert_eq!(
            resolve_entity_types_asset_path("assets/levels/planet1/level.json", "assets/levels/entity_types.json"),
            "levels/entity_types.json"
        );
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



