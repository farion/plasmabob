use std::collections::HashMap;
use serde_json::Value;

use bevy::asset::io::AssetSourceId;
use bevy::prelude::*;
use futures_lite::stream::StreamExt as _;
use serde::Deserialize;

const DEFAULT_ANIMATION_FRAME_MS: u64 = 500;
const DEFAULT_ENTITY_TYPES_PATH: &str = "entity_types";

#[derive(Debug, Clone)]
pub(crate) struct LevelDefinition {
    pub(crate) terrain: TerrainDefinition,
    pub(crate) music: String,
    pub(crate) quotes: Vec<String>,
    pub(crate) bounds: Option<LevelBoundsDefinition>,
    pub(crate) story: Option<LevelStoryDefinition>,
    pub(crate) entity_types: HashMap<String, EntityTypeDefinition>,
    pub(crate) entities: Vec<EntityDefinition>,
}

#[derive(Resource, Debug)]
pub(crate) struct CachedLevelDefinition {
    asset_path: String,
    loaded_level: Result<LevelDefinition, LoadLevelError>,
}

impl CachedLevelDefinition {
    pub(crate) fn empty() -> Self {
        Self {
            asset_path: String::new(),
            loaded_level: Err(LoadLevelError::NotLoaded("Level has not been loaded yet".to_string())),
        }
    }

    pub(crate) fn refresh(&mut self, asset_server: &AssetServer, asset_path: &str) {
        let asset_path = asset_path.trim().trim_start_matches("assets/");
        self.loaded_level = load_level_from_asset_server(asset_server, asset_path);
        self.asset_path = asset_path.to_string();
    }

    pub(crate) fn level_definition(&self) -> Result<&LevelDefinition, &LoadLevelError> {
        self.loaded_level.as_ref().map_err(|error| error)
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
    #[serde(default)]
    story: Option<LevelStoryDefinition>,
    #[serde(default = "default_entity_types_path")]
    entity_types_path: String,
    entities: Vec<EntityDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StorySlideDefinition {
    pub(crate) text: String,
    pub(crate) background: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LevelStoryDefinition {
    #[serde(default)]
    pub(crate) start: Option<StorySlideDefinition>,
    #[serde(default)]
    pub(crate) win: Option<StorySlideDefinition>,
    #[serde(default)]
    pub(crate) lose: Option<StorySlideDefinition>,
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
    pub(crate) fn terrain_background_asset_path(&self) -> &str {
        &self.terrain.background
    }

    pub(crate) fn music_asset_path(&self) -> &str {
        &self.music
    }

    pub(crate) fn quote_asset_paths(&self) -> &[String] {
        &self.quotes
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
pub(crate) struct EntityTypeStateDefinition {
    #[serde(default)]
    pub(crate) animation: Vec<String>,
    #[serde(default)]
    pub(crate) hitbox: Vec<[f32; 2]>,
    #[serde(default)]
    pub(crate) animation_frame_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EntityTypeDefinition {
    #[serde(rename = "component")]
    pub(crate) components: Vec<String>,

    #[serde(default)]
    pub(crate) states: HashMap<String, EntityTypeStateDefinition>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    #[serde(default)]
    pub(crate) health: Option<HealthDefinition>,
    #[serde(default)]
    pub(crate) damage: Option<i32>,
    #[serde(default)]
    pub(crate) effect_heal: Option<EffectHealDefinition>,
    #[serde(default)]
    pub(crate) melee_attack: Option<MeleeAttackDefinition>,
    #[serde(default)]
    pub(crate) range_attack: Option<RangeAttackDefinition>,
    /// Maximum range of the plasma beam (player only). Enables the PlasmaAttack component.
    #[serde(default)]
    pub(crate) attack_range: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EffectHealDefinition {
    #[serde(default)]
    pub(crate) heal: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MeleeAttackDefinition {
    #[serde(default)]
    pub(crate) damage: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RangeAttackDefinition {
    #[serde(default)]
    pub(crate) damage: Option<i32>,
    #[serde(default)]
    pub(crate) speed: Option<f32>,
    #[serde(default)]
    pub(crate) frequency: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HealthDefinition {
    /// The hp value for this entity type when present. Kept optional to allow
    /// per-entity overrides via the `<component>.<attribute>` override map.
    #[serde(default)]
    pub(crate) health: Option<i32>,
}

impl EntityTypeDefinition {
    fn all_state_names(&self) -> Vec<String> {
        let mut state_names = Vec::new();

        for state in self.states.keys() {
            if !state_names.iter().any(|name| name == state) {
                state_names.push(state.clone());
            }
        }

        state_names
    }

    fn state_hitbox_points(&self, state_name: &str) -> Vec<[f32; 2]> {
        if let Some(state) = self.states.get(state_name) {
            if !state.hitbox.is_empty() {
                return state.hitbox.clone();
            }
        }

        if let Some(default_state) = self.states.get("default") {
            if !default_state.hitbox.is_empty() {
                return default_state.hitbox.clone();
            }
        }

        Vec::new()
    }

    fn state_animation_frame_ms(&self, state_name: &str) -> u64 {
        self.states
            .get(state_name)
            .and_then(|state| state.animation_frame_ms)
            .or_else(|| {
                self.states
                    .get("default")
                    .and_then(|state| state.animation_frame_ms)
            })
            .unwrap_or(DEFAULT_ANIMATION_FRAME_MS)
    }

    pub(crate) fn normalized_animations(&self) -> HashMap<String, Vec<String>> {
        let mut animations = HashMap::new();

        for state_name in self.all_state_names() {
            let frames = self
                .states
                .get(&state_name)
                .map(|state| state.animation.clone())
                .unwrap_or_default();

            animations.insert(state_name, frames);
        }

        animations
    }

    pub(crate) fn default_animation_path(&self) -> Option<String> {
        let animations = self.normalized_animations();

        animations
            .get("default")
            .and_then(|frames| frames.first())
            .cloned()
            .or_else(|| animations.values().flat_map(|frames| frames.iter()).next().cloned())
    }

    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub(crate) fn centered_hitbox_polygon(&self) -> Result<Vec<Vec2>, EntityTypeError> {
        self.centered_hitbox_polygon_for_state("default")
    }

    pub(crate) fn centered_hitbox_polygon_for_state(&self, state_name: &str) -> Result<Vec<Vec2>, EntityTypeError> {
        let state_hitbox = self.state_hitbox_points(state_name);
        let points: Vec<[f32; 2]> = if state_hitbox.is_empty() {
            vec![
                [0.0, 0.0],
                [self.width, 0.0],
                [self.width, self.height],
                [0.0, self.height],
            ]
        } else {
            state_hitbox
        };

        if points.len() < 3 {
            return Err(EntityTypeError::InvalidHitbox("hitbox polygon requires at least 3 points".to_string()));
        }

        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;

        Ok(points
            .iter()
            .map(|point| Vec2::new(point[0] - half_width, point[1] - half_height))
            .collect())
    }

    pub(crate) fn animation_frame_seconds(&self) -> f32 {
        self.animation_frame_seconds_for_state("default")
    }

    pub(crate) fn animation_frame_seconds_for_state(&self, state_name: &str) -> f32 {
        let interval_ms = self.state_animation_frame_ms(state_name);
        (interval_ms as f32 / 1000.0).max(0.001)
    }

    pub(crate) fn animation_frame_seconds_by_state(&self) -> HashMap<String, f32> {
        let mut durations = HashMap::new();

        for state_name in self.all_state_names() {
            durations.insert(
                state_name.clone(),
                self.animation_frame_seconds_for_state(&state_name),
            );
        }

        durations
    }

    pub(crate) fn centered_hitbox_polygons_by_state(&self) -> Result<HashMap<String, Vec<Vec2>>, EntityTypeError> {
        let mut hitboxes = HashMap::new();

        for state_name in self.all_state_names() {
            hitboxes.insert(
                state_name.clone(),
                self.centered_hitbox_polygon_for_state(&state_name)?,
            );
        }

        Ok(hitboxes)
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
    /// Per-instance override attributes. Keys use the pattern `<component>.<attribute>`,
    /// for example `effect_heal.heal`.
    #[serde(flatten)]
    pub(crate) overrides: HashMap<String, Value>,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum LoadLevelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("{0}")]
    NotLoaded(String),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum EntityTypeError {
    #[error("Invalid hitbox: {0}")]
    InvalidHitbox(String),
}

pub(crate) fn load_level_from_asset_server(
    asset_server: &AssetServer,
    asset_path: &str,
) -> Result<LevelDefinition, LoadLevelError> {
    let content = crate::helper::asset_io::read_asset_text(asset_server, asset_path)?;
    let raw_level: RawLevelDefinition = serde_json::from_str(&content)?;

    let entity_types_dir = raw_level.entity_types_path.clone();
    let entity_types = load_entity_types_from_dir(asset_server, &entity_types_dir)?;

    Ok(LevelDefinition {
        terrain: raw_level.terrain,
        music: raw_level.music,
        quotes: raw_level.quotes,
        bounds: raw_level.bounds,
        story: raw_level.story,
        entity_types,
        entities: raw_level.entities,
    })
}

// read_asset_text_from_server moved to `src/helper/asset_io.rs` as a shared helper.

/// Loads all `*.json` files from a directory via the AssetServer. The filename stem becomes the entity-type key.
fn load_entity_types_from_dir(
    asset_server: &AssetServer,
    dir_asset_path: &str,
) -> Result<HashMap<String, EntityTypeDefinition>, LoadLevelError> {
    let source = asset_server.get_source(AssetSourceId::Default).map_err(|error| {
        LoadLevelError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Asset source error: {error}"),
        ))
    })?;

    let paths: Vec<_> = pollster::block_on(async {
        let mut stream = source
            .reader()
            .read_directory(dir_asset_path.as_ref())
            .await
            .map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Could not list asset directory '{dir_asset_path}': {error}"),
                )
            })?;

        let mut paths = Vec::new();
        while let Some(path) = stream.next().await {
            paths.push(path);
        }
        Ok::<_, std::io::Error>(paths)
    })?;

    let mut entity_types = HashMap::new();
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let content = crate::helper::asset_io::read_asset_text(asset_server, &path.to_string_lossy())?;
        let definition: EntityTypeDefinition = serde_json::from_str(&content)?;
        validate_entity_type_definition(&definition, &key)?;
        entity_types.insert(key, definition);
    }

    Ok(entity_types)
}

fn validate_entity_type_definition(
    definition: &EntityTypeDefinition,
    entity_type_name: &str,
) -> Result<(), LoadLevelError> {
    if definition.states.is_empty() {
        return Err(LoadLevelError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("entity type '{entity_type_name}' requires a non-empty 'states' object"),
        )));
    }

    if !definition.states.contains_key("default") {
        return Err(LoadLevelError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("entity type '{entity_type_name}' requires a 'states.default' definition"),
        )));
    }

    Ok(())
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
    use bevy::asset::AssetPlugin;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root() -> String {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let tmp_dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        format!("{tmp_dir}/plasmabob-tests-{unique}")
    }

    fn write_temp_file(root: &str, relative_path: &str, content: &str) {
        let full_path = format!("{root}/{relative_path}");

        std::fs::create_dir_all(
            std::path::Path::new(&full_path)
                .parent()
                .expect("temporary file path should have a parent directory"),
        )
        .expect("temporary directory should be created");

        std::fs::write(full_path, content).expect("temporary file should be written");
    }

    fn with_temp_asset_root<F: FnOnce(&str)>(test: F) {
        struct TempRootGuard {
            root: String,
        }

        impl Drop for TempRootGuard {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.root);
            }
        }

        let root = unique_temp_root();
        std::fs::create_dir_all(format!("{root}/assets")).expect("temporary assets directory should exist");
        let _guard = TempRootGuard { root: root.clone() };
        test(&root);
    }

    fn with_test_asset_server<F: FnOnce(&AssetServer)>(root: &str, test: F) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin {
            file_path: format!("{root}/assets"),
            ..default()
        });
        let asset_server = app.world().resource::<AssetServer>().clone();
        test(&asset_server);
    }

    #[test]
    fn parses_the_split_level_schema() {
        with_temp_asset_root(|root| {
            write_temp_file(
                root,
                "assets/levels/level.json",
                r#"
                {
                    "terrain": {
                        "background": "backgrounds/level1.png"
                    },
                    "music": "music/level1.ogg",
                    "bounds": {
                        "width": 1584,
                        "height": 1024
                    },
                    "entity_types_path": "entity_types",
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
                root,
                "assets/entity_types/dirt.json",
                r#"{
                    "component": ["floor"],
                    "states": {
                        "default": {
                            "animation": ["dirt/default1.png", "dirt/default2.png"],
                            "animation_frame_ms": 500
                        }
                    },
                    "width": 100,
                    "height": 20
                }"#,
            );
            write_temp_file(
                root,
                "assets/entity_types/cockroach.json",
                r#"{
                    "component": ["npc", "hostile"],
                    "disposition": "hostile",
                    "states": {
                        "default": { "animation": ["cockroach/default1.png", "cockroach/default2.png"] },
                        "walk": { "animation": [] },
                        "jump": { "animation": [] },
                        "die": { "animation": [] },
                        "hit": { "animation": [] },
                        "fight": { "animation": [] }
                    },
                    "width": 100,
                    "height": 20
                }"#,
            );
            write_temp_file(
                root,
                "assets/entity_types/bob.json",
                r#"{
                    "component": ["player"],
                    "states": {
                        "default": {
                            "animation": ["bob/default1.png", "bob/default2.png"],
                            "animation_frame_ms": 250
                        },
                        "walk": { "animation": [] },
                        "jump": { "animation": [] },
                        "die": { "animation": [] },
                        "hit": { "animation": [] },
                        "fight": { "animation": [] }
                    },
                    "width": 100,
                    "height": 20
                }"#,
            );

            with_test_asset_server(root, |asset_server| {
                let parsed =
                    load_level_from_asset_server(asset_server, "levels/level.json").expect("schema should parse");

                assert_eq!(parsed.entity_types.len(), 3);
                assert_eq!(parsed.entities.len(), 2);
                assert_eq!(parsed.bounds_size(), Some(Vec2::new(1584.0, 1024.0)));
                assert_eq!(parsed.terrain_background_asset_path(), "backgrounds/level1.png");
                assert_eq!(parsed.music_asset_path(), "music/level1.ogg");
                assert!(parsed.quote_asset_paths().is_empty());
                assert_eq!(parsed.entity_types["dirt"].components, vec!["floor"]);
                // `disposition` field is removed; ensure the cockroach type still parsed and
                // that it includes the "hostile" component in its components list.
                assert!(parsed.entity_types["cockroach"].components.iter().any(|c| c == "hostile"));
                assert_eq!(parsed.entity_types["bob"].width, 100.0);
                assert_eq!(parsed.entity_types["bob"].animation_frame_seconds_for_state("default"), 0.25);
                assert_eq!(parsed.entities[1].z_index, Some(20.0));
            });
        });
    }

    #[test]
    fn uses_default_entity_types_directory_when_field_is_missing() {
        with_temp_asset_root(|root| {
            write_temp_file(
                root,
                "assets/levels/level.json",
                r#"
                {
                    "terrain": { "background": "backgrounds/level1.png" },
                    "music": "music/level1.ogg",
                    "entities": [
                        { "id": "dummy1", "entity_type": "dummy", "x": 0, "y": 0 }
                    ]
                }
                "#,
            );

            write_temp_file(
                root,
                "assets/entity_types/dummy.json",
                r#"{
                    "component": ["npc"],
                    "states": {
                        "default": { "animation": ["dirt/default1.png"] }
                    },
                    "width": 16,
                    "height": 16
                }"#,
            );

            with_test_asset_server(root, |asset_server| {
                let parsed =
                    load_level_from_asset_server(asset_server, "levels/level.json").expect("schema should parse");
                assert_eq!(parsed.entity_types["dummy"].animation_frame_seconds(), 0.5);
            });
        });
    }

    #[test]
    fn parses_optional_level_story_sections() {
        with_temp_asset_root(|root| {
            write_temp_file(
                root,
                "assets/levels/level.json",
                r#"
                {
                    "terrain": { "background": "backgrounds/level1.png" },
                    "music": "music/level1.ogg",
                    "story": {
                        "start": {
                            "text": "story/level_start.md",
                            "background": "backgrounds/level1.png"
                        },
                        "win": {
                            "text": "story/level_win.md",
                            "background": "backgrounds/level1.png"
                        },
                        "lose": {
                            "text": "story/level_lose.md",
                            "background": "backgrounds/level1.png"
                        }
                    },
                    "entities": [
                        { "id": "dummy1", "entity_type": "dummy", "x": 0, "y": 0 }
                    ]
                }
                "#,
            );

            write_temp_file(
                root,
                "assets/entity_types/dummy.json",
                r#"{
                    "component": ["npc"],
                    "states": {
                        "default": { "animation": ["dummy/default.png"] }
                    },
                    "width": 16,
                    "height": 16
                }"#,
            );

            with_test_asset_server(root, |asset_server| {
                let parsed =
                    load_level_from_asset_server(asset_server, "levels/level.json").expect("schema should parse");

                let story = parsed.story.as_ref().expect("story should exist");
                assert_eq!(story.start.as_ref().expect("start story").text, "story/level_start.md");
                assert_eq!(story.win.as_ref().expect("win story").text, "story/level_win.md");
                assert_eq!(story.lose.as_ref().expect("lose story").text, "story/level_lose.md");
            });
        });
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



