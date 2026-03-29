use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::model::{normalize_asset_reference, EntityDefinition, EntityTypeDefinition, LevelFile};

#[derive(Debug, Clone)]
pub(crate) struct LevelEntry {
    pub(crate) display_name: String,
    pub(crate) asset_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedLevel {
    pub(crate) level_asset_path: String,
    pub(crate) level_fs_path: PathBuf,
    pub(crate) level: LevelFile,
    pub(crate) entity_types: HashMap<String, EntityTypeDefinition>,
}

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live directly inside workspace root")
        .to_path_buf()
}

pub(crate) fn assets_dir() -> PathBuf {
    workspace_root().join("assets")
}

pub(crate) fn levels_dir() -> PathBuf {
    assets_dir().join("levels")
}

pub(crate) fn asset_path_to_filesystem_path(asset_path: &str) -> PathBuf {
    assets_dir().join(normalize_asset_reference(asset_path))
}

pub(crate) fn scan_levels() -> Result<Vec<LevelEntry>, String> {
    scan_levels_in_dir(&levels_dir())
}

fn scan_levels_in_dir(levels_dir: &Path) -> Result<Vec<LevelEntry>, String> {
    let mut levels = Vec::new();
    let assets_root = levels_dir
        .parent()
        .ok_or_else(|| "levels directory has no parent".to_string())?;

    for entry in std::fs::read_dir(levels_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        if serde_json::from_str::<LevelFile>(&content).is_err() {
            continue;
        }

        let Ok(relative) = path.strip_prefix(assets_root) else {
            continue;
        };

        levels.push(LevelEntry {
            display_name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unbekannt")
                .to_string(),
            asset_path: relative.to_string_lossy().replace('\\', "/"),
        });
    }

    levels.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    Ok(levels)
}

pub(crate) fn load_level(level_asset_path: &str) -> Result<LoadedLevel, String> {
    let level_fs_path = asset_path_to_filesystem_path(level_asset_path);
    let content = std::fs::read_to_string(&level_fs_path).map_err(|error| error.to_string())?;
    let level: LevelFile = serde_json::from_str(&content).map_err(|error| error.to_string())?;

    let entity_types_dir = find_entity_types_dir(level_asset_path, &level.entity_types_path)?;
    let entity_types = load_entity_types_from_dir(&entity_types_dir)?;

    Ok(LoadedLevel {
        level_asset_path: normalize_asset_reference(level_asset_path),
        level_fs_path,
        level,
        entity_types,
    })
}

fn find_entity_types_dir(level_asset_path: &str, configured_path: &str) -> Result<PathBuf, String> {
    let candidates = resolve_entity_type_dir_candidates(level_asset_path, configured_path);

    for candidate in &candidates {
        if candidate.is_dir() {
            return Ok(candidate.clone());
        }
    }

    let checked: Vec<String> = candidates
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    Err(format!(
        "Entity-Type-Verzeichnis nicht gefunden. Geprüft: {}",
        checked.join(", ")
    ))
}

fn resolve_entity_type_dir_candidates(level_asset_path: &str, configured_path: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();
    let assets_root = assets_dir();
    let normalized_level_asset = normalize_asset_reference(level_asset_path);
    let normalized_configured = normalize_asset_reference(configured_path);
    let sanitized = normalized_configured.trim_end_matches('/');

    add_unique_candidate_path(&mut candidates, assets_root.join(sanitized));

    if sanitized.ends_with(".json") {
        if let Some(stem) = Path::new(sanitized).file_stem().and_then(|value| value.to_str()) {
            add_unique_candidate_path(&mut candidates, assets_root.join(stem));
            if stem == "entity_types" {
                add_unique_candidate_path(&mut candidates, assets_root.join("entitytypes"));
            }
        }
    }

    if !sanitized.contains('/') {
        let level_directory = Path::new(&normalized_level_asset)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        add_unique_candidate_path(&mut candidates, assets_root.join(level_directory).join(sanitized));
    }

    add_unique_candidate_path(&mut candidates, assets_root.join("entitytypes"));
    add_unique_candidate_path(&mut candidates, assets_root.join("entity_types"));

    candidates
}

fn add_unique_candidate_path(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.iter().any(|path| path == &candidate) {
        candidates.push(candidate);
    }
}

fn load_entity_types_from_dir(dir_path: &Path) -> Result<HashMap<String, EntityTypeDefinition>, String> {
    let mut entity_types = HashMap::new();

    for entry in std::fs::read_dir(dir_path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let key = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string();
        if key.is_empty() {
            continue;
        }

        let content = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let definition = serde_json::from_str::<EntityTypeDefinition>(&content)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_entity_type_definition(&definition, &key, &path)?;
        entity_types.insert(key, definition);
    }

    Ok(entity_types)
}

fn validate_entity_type_definition(
    definition: &EntityTypeDefinition,
    key: &str,
    path: &Path,
) -> Result<(), String> {
    if definition.states.is_empty() {
        return Err(format!(
            "{}: entity type '{key}' requires a non-empty 'states' object",
            path.display()
        ));
    }

    if !definition.states.contains_key("default") {
        return Err(format!(
            "{}: entity type '{key}' requires a 'states.default' definition",
            path.display()
        ));
    }

    Ok(())
}

pub(crate) fn save_level(level_fs_path: &Path, level: &LevelFile) -> Result<(), String> {
    let content = serde_json::to_string_pretty(level).map_err(|error| error.to_string())?;
    std::fs::write(level_fs_path, format!("{content}\n")).map_err(|error| error.to_string())
}

pub(crate) fn next_entity_id(entity_type: &str, entities: &[EntityDefinition]) -> String {
    let max_suffix = entities
        .iter()
        .filter_map(|entity| entity.id.strip_prefix(entity_type))
        .filter_map(|suffix| suffix.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    format!("{entity_type}{}", max_suffix + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("plasmabob-editor-tests-{unique}"))
    }

    fn write_file(root: &Path, relative_path: &str, content: &str) -> PathBuf {
        let path = root.join(relative_path);
        std::fs::create_dir_all(path.parent().expect("file should have a parent"))
            .expect("parent directory should be creatable");
        std::fs::write(&path, content).expect("test file should be writable");
        path
    }

    #[test]
    fn next_entity_id_uses_highest_numeric_suffix() {
        let entities = vec![
            EntityDefinition {
                id: "cockroach1".to_string(),
                entity_type: "cockroach".to_string(),
                x: 0.0,
                y: 0.0,
                z_index: None,
            },
            EntityDefinition {
                id: "cockroach9".to_string(),
                entity_type: "cockroach".to_string(),
                x: 0.0,
                y: 0.0,
                z_index: None,
            },
        ];

        assert_eq!(next_entity_id("cockroach", &entities), "cockroach10");
    }

    #[test]
    fn scan_levels_skips_non_level_json_files() {
        let root = unique_temp_root();
        let levels_path = root.join("assets/levels");

        write_file(
            &root,
            "assets/levels/level1.json",
            r#"{
                "terrain": { "background": "assets/backgrounds/level1.png" },
                "music": "assets/music/level1.ogg",
                "entity_types_path": "entity_types",
                "entities": []
            }"#,
        );
        write_file(
            &root,
            "assets/levels/not_a_level.json",
            r#"{ "bob": { "component": ["player"], "states": { "default": { "animation": [] } }, "width": 1, "height": 1 } }"#,
        );

        let levels = scan_levels_in_dir(&levels_path).expect("scan should succeed");
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].display_name, "level1.json");
    }

    #[test]
    fn loads_entity_types_from_split_directory() {
        let root = unique_temp_root();
        let dir = root.join("assets/entity_types");

        write_file(
            &root,
            "assets/entity_types/bob.json",
            r#"{
                "component": ["player"],
                "states": {
                    "default": {
                        "animation": ["assets/bob/Bob-Stand.png"],
                        "animation_frame_ms": 500
                    }
                },
                "width": 16,
                "height": 16
            }"#,
        );

        let entity_types = load_entity_types_from_dir(&dir).expect("load should succeed");
        assert!(entity_types.contains_key("bob"));
    }

    #[test]
    fn resolves_entity_type_dir_candidates_with_fallbacks() {
        let root = unique_temp_root();
        let level_path = root.join("assets/levels/level1.json");
        let level_asset = level_path
            .strip_prefix(root.join("assets"))
            .expect("path should be below assets")
            .to_string_lossy()
            .replace('\\', "/");

        let candidates = resolve_entity_type_dir_candidates(&level_asset, "entity_types.json");
        let as_strings: Vec<String> = candidates
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();

        assert!(as_strings.iter().any(|path| path.ends_with("/assets/entity_types")));
        assert!(as_strings.iter().any(|path| path.ends_with("/assets/entitytypes")));
    }
}



