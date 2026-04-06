use bevy::asset::io::AssetSourceId;
use bevy::prelude::*;
use futures_lite::stream::StreamExt as _;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldDefinition {
    pub(crate) name: String,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) background: String,
    #[serde(default)]
    pub(crate) story: Option<WorldStoryDefinition>,
    #[serde(default)]
    pub(crate) planets: Vec<PlanetDefinition>,
}

impl WorldDefinition {
    pub(crate) fn virtual_size(&self) -> Vec2 {
        Vec2::new(self.width.max(1.0), self.height.max(1.0))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldStorySlideDefinition {
    pub(crate) text: String,
    pub(crate) background: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldStoryDefinition {
    #[serde(default)]
    pub(crate) start: Option<WorldStorySlideDefinition>,
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) win: Option<WorldStorySlideDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PlanetDefinition {
    pub(crate) name: String,
    pub(crate) position: [f32; 2],
    pub(crate) radius: f32,
    pub(crate) color: [u8; 3],
    #[serde(default)]
    pub(crate) levels: Vec<PlanetLevelDefinition>,
}

impl PlanetDefinition {
    pub(crate) fn position_vec2(&self) -> Vec2 {
        Vec2::new(self.position[0], self.position[1])
    }

    pub(crate) fn color_vec3(&self) -> Vec3 {
        Vec3::new(
            self.color[0] as f32 / 255.0,
            self.color[1] as f32 / 255.0,
            self.color[2] as f32 / 255.0,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PlanetLevelDefinition {
    pub(crate) name: String,
    pub(crate) json: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WorldCatalogEntry {
    pub(crate) asset_path: String,
    pub(crate) definition: WorldDefinition,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct WorldCatalog {
    worlds: Vec<WorldCatalogEntry>,
    last_error: Option<LoadWorldError>,
}

impl WorldCatalog {
    pub(crate) fn worlds(&self) -> &[WorldCatalogEntry] {
        &self.worlds
    }

    pub(crate) fn world(&self, index: usize) -> Option<&WorldCatalogEntry> {
        self.worlds.get(index)
    }

    pub(crate) fn last_error(&self) -> Option<&LoadWorldError> {
        self.last_error.as_ref()
    }

    pub(crate) fn refresh(&mut self, asset_server: &AssetServer) {
        match load_world_catalog(asset_server, "worlds") {
            Ok(worlds) => {
                self.worlds = worlds;
                self.last_error = None;
            }
            Err(error) => {
                self.worlds.clear();
                self.last_error = Some(error);
            }
        }
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum LoadWorldError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

fn load_world_catalog(
    asset_server: &AssetServer,
    dir_asset_path: &str,
) -> Result<Vec<WorldCatalogEntry>, LoadWorldError> {
    let source = asset_server
        .get_source(AssetSourceId::Default)
        .map_err(|error| {
            LoadWorldError::Io(std::io::Error::new(
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

    let mut worlds = Vec::new();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let asset_path = path.to_string_lossy().to_string();
        let content = crate::helper::asset_io::read_asset_text(asset_server, &asset_path)?;
        let definition: WorldDefinition = serde_json::from_str(&content)?;

        worlds.push(WorldCatalogEntry {
            asset_path,
            definition,
        });
    }

    worlds.sort_by(|left, right| {
        left.definition
            .name
            .to_lowercase()
            .cmp(&right.definition.name.to_lowercase())
    });
    Ok(worlds)
}

// read_asset_text_from_server moved to `src/helper/asset_io.rs` as a shared helper.

pub(crate) fn find_directional_neighbor(
    planets: &[PlanetDefinition],
    current_index: usize,
    direction: Vec2,
) -> Option<usize> {
    let Some(current) = planets.get(current_index) else {
        return None;
    };

    if direction.length_squared() <= f32::EPSILON {
        return None;
    }

    let direction = direction.normalize();
    let current_position = current.position_vec2();

    let mut best: Option<(usize, f32)> = None;

    for (candidate_index, candidate) in planets.iter().enumerate() {
        if candidate_index == current_index {
            continue;
        }

        let delta = candidate.position_vec2() - current_position;
        let distance = delta.length();
        if distance <= f32::EPSILON {
            continue;
        }

        let delta_dir = delta / distance;
        let alignment = direction.dot(delta_dir);
        if alignment <= 0.15 {
            continue;
        }

        // Prefer planets in the requested direction first, then nearest among them.
        let score = (1.0 - alignment) * 1000.0 + distance;

        match best {
            Some((_, best_score)) if score >= best_score => {}
            _ => best = Some((candidate_index, score)),
        }
    }

    best.map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planet(name: &str, x: f32, y: f32) -> PlanetDefinition {
        PlanetDefinition {
            name: name.to_string(),
            position: [x, y],
            radius: 16.0,
            color: [255, 255, 255],
            levels: Vec::new(),
        }
    }

    #[test]
    fn picks_right_neighbor_by_direction() {
        let planets = vec![
            planet("center", 100.0, 100.0),
            planet("left", 10.0, 100.0),
            planet("right", 200.0, 100.0),
            planet("up", 100.0, 200.0),
        ];

        let next = find_directional_neighbor(&planets, 0, Vec2::X);
        assert_eq!(next, Some(2));
    }

    #[test]
    fn returns_none_when_no_neighbor_in_direction() {
        let planets = vec![planet("a", 0.0, 0.0), planet("b", 100.0, 0.0)];

        let next = find_directional_neighbor(&planets, 1, Vec2::X);
        assert_eq!(next, None);
    }

    #[test]
    fn deserializes_world_virtual_size() {
        let json = r#"
        {
            "name": "Auralis",
            "width": 320,
            "height": 239,
            "background": "worlds/auralis.png",
            "planets": [],
            "paths": []
        }
        "#;

        let world: WorldDefinition = serde_json::from_str(json).expect("world json should parse");
        assert_eq!(world.virtual_size(), Vec2::new(320.0, 239.0));
    }

    #[test]
    fn deserializes_planet_color_rgb() {
        let json = r#"
        {
            "name": "Auralis",
            "width": 320,
            "height": 239,
            "background": "worlds/auralis.png",
            "planets": [
                {
                    "name": "Viridara",
                    "position": [44, 91],
                    "radius": 35,
                    "color": [42, 200, 128],
                    "levels": []
                }
            ]
        }
        "#;

        let world: WorldDefinition = serde_json::from_str(json).expect("world json should parse");
        assert_eq!(world.planets[0].color, [42, 200, 128]);
    }

    #[test]
    fn fails_when_planet_color_is_missing() {
        let json = r#"
        {
            "name": "Auralis",
            "width": 320,
            "height": 239,
            "background": "worlds/auralis.png",
            "planets": [
                {
                    "name": "Viridara",
                    "position": [44, 91],
                    "radius": 35,
                    "levels": []
                }
            ]
        }
        "#;

        let result = serde_json::from_str::<WorldDefinition>(json);
        assert!(result.is_err(), "planet color must be required");
    }

    #[test]
    fn deserializes_optional_world_story() {
        let json = r#"
        {
            "name": "Auralis",
            "width": 320,
            "height": 239,
            "background": "worlds/auralis.png",
            "story": {
                "start": {
                    "text": "story/world_start.md",
                    "background": "worlds/auralis.png"
                },
                "win": {
                    "text": "story/world_win.md",
                    "background": "worlds/auralis.png"
                }
            },
            "planets": []
        }
        "#;

        let world: WorldDefinition = serde_json::from_str(json).expect("world json should parse");
        let story = world.story.as_ref().expect("story should be present");
        assert_eq!(
            story.start.as_ref().expect("start story").text,
            "story/world_start.md"
        );
        assert_eq!(
            story.win.as_ref().expect("win story").text,
            "story/world_win.md"
        );
    }
}
