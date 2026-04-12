use bevy::asset::io::AssetSourceId;
use bevy::prelude::*;
use futures_lite::stream::StreamExt as _;
use std::collections::HashMap;

use crate::game::level::errors::LoadLevelError;
use crate::game::level::types::{CachedLevelDefinition, EntityTypeDefinition, LevelDefinition, LevelEntity, PropValue};

/// Load a level and its entity types from the given asset path into a new
/// `CachedLevelDefinition`.
pub fn load_level_from_asset(
    asset_server: &AssetServer,
    asset_path: &str,
) -> Result<CachedLevelDefinition, LoadLevelError> {
    // Read level JSON text
    let content = crate::helper::asset_io::read_asset_text(asset_server, asset_path).map_err(LoadLevelError::Io)?;

    let level: LevelDefinition = serde_json::from_str(&content).map_err(LoadLevelError::Parse)?;

    // Determine entity types location (file or directory). Use fallback "entity_types" when absent.
    let entity_types_ref = level
        .entity_types_path
        .clone()
        .unwrap_or_else(|| "entity_types".to_string());

    // Attempt to load entity types. Support both single JSON file (entity_types.json)
    // and a directory containing multiple .json files.
    let mut entity_types_map: HashMap<String, EntityTypeDefinition> = HashMap::new();

    if entity_types_ref.to_ascii_lowercase().ends_with(".json") {
        // Single file containing a map/object of entity types or a single entity type.
        let txt = crate::helper::asset_io::read_asset_text(asset_server, &entity_types_ref);
        match txt {
            Ok(text) => {
                // Try parsing as a map of entity type name -> definition first
                let as_map: Result<HashMap<String, EntityTypeDefinition>, _> = serde_json::from_str(&text);
                if let Ok(map) = as_map {
                    entity_types_map.extend(map);
                } else {
                    // Fallback: parse as a single EntityTypeDefinition and derive key from filename
                    let single: EntityTypeDefinition = serde_json::from_str(&text).map_err(LoadLevelError::Parse)?;
                    if let Some(stem) = asset_path_stem(&entity_types_ref) {
                        entity_types_map.insert(stem.to_string(), single);
                    } else {
                        return Err(LoadLevelError::EntityTypes(format!("Could not determine key for entity types file '{}'", entity_types_ref)));
                    }
                }
            }
            Err(e) => return Err(LoadLevelError::Io(e)),
        }
    } else {
        // Treat as directory: list .json entries and load each
        let source = asset_server
            .get_source(AssetSourceId::Default)
            .map_err(|err| LoadLevelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Asset source error: {err}"))))?;

        let paths: Vec<std::path::PathBuf> = pollster::block_on(async {
            let mut stream = source
                .reader()
                .read_directory(entity_types_ref.as_ref())
                .await
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Could not list asset directory '{}': {error}", entity_types_ref)))?;

            let mut paths: Vec<std::path::PathBuf> = Vec::new();
            while let Some(path) = stream.next().await {
                paths.push(path);
            }
            Ok::<_, std::io::Error>(paths)
        })?;

        for path in paths {
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let asset_path = path.to_string_lossy().to_string();
            let txt = crate::helper::asset_io::read_asset_text(asset_server, &asset_path).map_err(LoadLevelError::Io)?;
            let et: EntityTypeDefinition = serde_json::from_str(&txt).map_err(LoadLevelError::Parse)?;
            if let Some(stem) = asset_path_stem(&asset_path) {
                entity_types_map.insert(stem.to_string(), et);
            }
        }
    }

    // Ensure entities are fully materialized (they are parsed already by serde into Vec<LevelEntity>)
    Ok(CachedLevelDefinition {
        asset_path: Some(asset_path.to_string()),
        level: Some(level),
        entity_types: entity_types_map,
        error: None,
    })
}

fn asset_path_stem(path: &str) -> Option<&str> {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
}

