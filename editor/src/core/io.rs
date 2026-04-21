use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::core::{
    normalize_asset_reference, EntityDefinition, EntityTypeDefinition, LevelFile,
    StateMachineDefinition,
};

#[derive(Debug, Clone)]
pub struct LevelEntry {
    pub display_name: String,
    pub asset_path: String,
}

#[derive(Debug, Clone)]
pub struct WorldEntry {
    pub display_name: String,
    pub asset_path: String,
}

#[derive(Debug, Clone)]
pub struct LoadedLevel {
    pub level_asset_path: String,
    pub level_fs_path: PathBuf,
    pub level: LevelFile,
    pub entity_types: HashMap<String, EntityTypeDefinition>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityTypeSyncReport {
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
}

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    frame_index: u32,
    asset_path: String,
    filesystem_path: PathBuf,
}

const SCALE_FACTOR: f64 = 7.48;

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live directly inside workspace root")
        .to_path_buf()
}

pub fn assets_dir() -> PathBuf {
    workspace_root().join("assets")
}

pub fn worlds_dir() -> PathBuf {
    assets_dir().join("worlds")
}

pub fn asset_path_to_filesystem_path(asset_path: &str) -> PathBuf {
    assets_dir().join(normalize_asset_reference(asset_path))
}

// Scan the source tree for available gameplay component names.
// Returns file stems (e.g. "player", "effect_heal") found under src/game/components.
pub fn scan_game_components() -> Result<Vec<String>, String> {
    let components_dir = workspace_root().join("src").join("game").join("components");
    let mut names = Vec::new();

    let entries = std::fs::read_dir(&components_dir)
        .map_err(|e| format!("{}: {e}", components_dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext != "rs" {
                continue;
            }
        } else {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem == "mod" {
                continue;
            }
            names.push(stem.to_string());
        }
    }

    names.sort();
    Ok(names)
}

// Overwrite the `components` object keys in the entity-type JSON file for `entity_type_name`.
// Existing component payloads are preserved when the key stays present.
pub fn save_entity_type_components(
    entity_type_name: &str,
    components: &[String],
) -> Result<(), String> {
    let json_path = assets_dir()
        .join("entity_types")
        .join(format!("{entity_type_name}.json"));

    let content = std::fs::read_to_string(&json_path)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;
    let mut definition = serde_json::from_str::<EntityTypeDefinition>(&content)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;

    definition.set_component_names(components);

    save_entity_type_definition(entity_type_name, &definition)
}

pub fn save_entity_type_definition(
    entity_type_name: &str,
    definition: &EntityTypeDefinition,
) -> Result<(), String> {
    let json_path = assets_dir()
        .join("entity_types")
        .join(format!("{entity_type_name}.json"));
    let content = serde_json::to_string_pretty(definition).map_err(|error| error.to_string())?;
    std::fs::write(&json_path, format!("{content}\n"))
        .map_err(|error| format!("{}: {error}", json_path.display()))
}

pub fn scan_levels() -> Result<Vec<LevelEntry>, String> {
    scan_levels_in_dir(&worlds_dir())
}

pub fn scan_worlds() -> Result<Vec<WorldEntry>, String> {
    let mut worlds = Vec::new();
    let worlds_dir = worlds_dir();
    let assets_root = worlds_dir
        .parent()
        .ok_or_else(|| "worlds directory has no parent".to_string())?;

    for entry in std::fs::read_dir(&worlds_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
        {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            // parse minimal JSON to check validity and extract optional name
            if serde_json::from_str::<Value>(&content).is_err() {
                continue;
            }

            if let Ok(relative) = path.strip_prefix(assets_root) {
                let asset_path = relative.to_string_lossy().replace('\\', "/");
                // prefer `name` field from JSON
                let display_name = serde_json::from_str::<Value>(&content)
                    .ok()
                    .and_then(|v| v.get("name").and_then(Value::as_str).map(|s| s.to_string()))
                    .or_else(|| {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| asset_path.clone());

                worlds.push(WorldEntry {
                    display_name,
                    asset_path,
                });
            }
        }
    }

    worlds.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(worlds)
}

fn scan_levels_in_dir(worlds_dir: &Path) -> Result<Vec<LevelEntry>, String> {
    let mut levels = Vec::new();
    let assets_root = worlds_dir
        .parent()
        .ok_or_else(|| "worlds directory has no parent".to_string())?;

    for world_entry in std::fs::read_dir(worlds_dir).map_err(|error| error.to_string())? {
        let world_entry = world_entry.map_err(|error| error.to_string())?;
        let world_path = world_entry.path();
        if !world_path.is_dir() {
            continue;
        }

        for entry in std::fs::read_dir(&world_path).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if !is_world_level_file_name(&path) {
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

            let asset_path = relative.to_string_lossy().replace('\\', "/");
            levels.push(LevelEntry {
                display_name: asset_path.clone(),
                asset_path,
            });
        }
    }

    levels.sort_by(|left, right| {
        let left_key = level_sort_key(&left.display_name);
        let right_key = level_sort_key(&right.display_name);
        left_key
            .cmp(&right_key)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(levels)
}

fn level_sort_key(display_name: &str) -> (String, String, Option<u32>) {
    let path = Path::new(display_name);
    let world_dir = path
        .parent()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let (prefix, number) = split_level_prefix_and_number(&stem);
    (world_dir, prefix, number)
}

fn split_level_prefix_and_number(stem: &str) -> (String, Option<u32>) {
    let Some(level_index) = stem.find("level") else {
        return (stem.to_string(), None);
    };

    let prefix = stem[..level_index + "level".len()].to_string();
    let digits: String = stem[level_index + "level".len()..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();

    let level_number = if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    };

    (prefix, level_number)
}

fn is_world_level_file_name(path: &Path) -> bool {
    let is_json = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    if !is_json {
        return false;
    }

    let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
        return false;
    };

    let stem = stem.to_ascii_lowercase();
    let Some(level_index) = stem.find("level") else {
        return false;
    };

    stem[level_index + "level".len()..]
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
}

pub fn load_level(level_asset_path: &str) -> Result<LoadedLevel, String> {
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
        "Entity types directory not found. Checked: {}",
        checked.join(", ")
    ))
}

fn resolve_entity_type_dir_candidates(
    level_asset_path: &str,
    configured_path: &str,
) -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();
    let assets_root = assets_dir();
    let normalized_level_asset = normalize_asset_reference(level_asset_path);
    let normalized_configured = normalize_asset_reference(configured_path);
    let sanitized = normalized_configured.trim_end_matches('/');

    add_unique_candidate_path(&mut candidates, assets_root.join(sanitized));

    if sanitized.ends_with(".json") {
        if let Some(stem) = Path::new(sanitized)
            .file_stem()
            .and_then(|value| value.to_str())
        {
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
        add_unique_candidate_path(
            &mut candidates,
            assets_root.join(level_directory).join(sanitized),
        );
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

fn load_entity_types_from_dir(
    dir_path: &Path,
) -> Result<HashMap<String, EntityTypeDefinition>, String> {
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
        let mut definition = serde_json::from_str::<EntityTypeDefinition>(&content)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        definition.key = key.clone();
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
    let Some(state_machine) = definition.state_machine() else {
        return Err(format!(
            "{}: entity type '{key}' requires components.state_machine",
            path.display()
        ));
    };

    if state_machine.states.is_empty() {
        return Err(format!(
            "{}: entity type '{key}' requires a non-empty 'components.state_machine.states' object",
            path.display()
        ));
    }

    if state_machine.initial_state.is_empty() {
        return Err(format!(
            "{}: entity type '{key}' requires a non-empty 'components.state_machine.initial_state'",
            path.display()
        ));
    }

    if !state_machine
        .states
        .contains_key(&state_machine.initial_state)
    {
        return Err(format!(
            "{}: entity type '{key}' initial_state '{}' must exist in components.state_machine.states",
            path.display(),
            state_machine.initial_state
        ));
    }

    Ok(())
}

pub fn save_level(level_fs_path: &Path, level: &LevelFile) -> Result<(), String> {
    let content = serde_json::to_string_pretty(level).map_err(|error| error.to_string())?;
    std::fs::write(level_fs_path, format!("{content}\n")).map_err(|error| error.to_string())
}

pub(crate) fn save_entity_type_hitboxes(
    entity_type_name: &str,
    hitboxes_by_state: &HashMap<String, [[f32; 2]; 4]>,
) -> Result<(), String> {
    let json_path = assets_dir()
        .join("entity_types")
        .join(format!("{entity_type_name}.json"));

    let content = std::fs::read_to_string(&json_path)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;
    let mut definition = serde_json::from_str::<EntityTypeDefinition>(&content)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;

    let mut state_machine = definition.state_machine().ok_or_else(|| {
        format!(
            "{}: missing object 'components.state_machine'",
            json_path.display()
        )
    })?;

    for (state_key, points) in hitboxes_by_state {
        let hitbox_points = points
            .iter()
            .map(|[x, y]| [x.round().max(0.0), y.round().max(0.0)])
            .collect::<Vec<_>>();

        let state = state_machine.states.get_mut(state_key).ok_or_else(|| {
            format!(
                "{}: missing object 'components.state_machine.states.{state_key}'",
                json_path.display()
            )
        })?;
        state.collider_box = Some(hitbox_points);
    }

    definition
        .set_state_machine(state_machine)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;

    save_entity_type_definition(entity_type_name, &definition)
}

pub fn next_entity_id(entity_type: &str, entities: &[EntityDefinition]) -> String {
    let max_suffix = entities
        .iter()
        .filter_map(|entity| entity.id.strip_prefix(entity_type))
        .filter_map(|suffix| suffix.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    format!("{entity_type}{}", max_suffix + 1)
}

pub fn sync_entity_types_with_sprites() -> Result<EntityTypeSyncReport, String> {
    sync_entity_types_with_paths(
        &assets_dir().join("sprites"),
        &assets_dir().join("entity_types"),
    )
}

fn sync_entity_types_with_paths(
    sprites_dir: &Path,
    entity_types_dir: &Path,
) -> Result<EntityTypeSyncReport, String> {
    std::fs::create_dir_all(entity_types_dir).map_err(|error| error.to_string())?;

    let sprite_directories = scan_sprite_directories(sprites_dir)?;
    let sprite_names: HashSet<String> = sprite_directories.keys().cloned().collect();
    let existing_json_files = scan_entity_type_jsons(entity_types_dir)?;
    let mut report = EntityTypeSyncReport::default();

    for (json_name, json_path) in &existing_json_files {
        if !sprite_names.contains(json_name) {
            std::fs::remove_file(json_path).map_err(|error| error.to_string())?;
            report.deleted += 1;
        }
    }

    // Exclude a small set of test/demo sprite dirs that should not
    // produce entity-type JSON files (e.g. bob/betty). Keep this list
    // intentionally minimal and explicit.
    let excluded_names = ["bob", "betty"];
    for (entity_name, sprite_dir) in sprite_directories {
        if excluded_names.contains(&entity_name.as_str()) {
            println!(
                "Update Entity Types: skipping excluded sprite dir '{}'",
                entity_name
            );
            continue;
        }
        println!("Update Entity Types: processing '{}'", entity_name);
        let json_path = entity_types_dir.join(format!("{entity_name}.json"));
        let existing_json = if json_path.exists() {
            Some(
                std::fs::read_to_string(&json_path)
                    .map_err(|error| format!("{}: {error}", json_path.display()))?,
            )
        } else {
            None
        };

        let existing_root = match &existing_json {
            Some(content) => serde_json::from_str::<Value>(content)
                .map_err(|error| format!("{}: {error}", json_path.display()))?,
            None => Value::Object(Map::new()),
        };

        let merged_root = build_entity_type_json(&entity_name, &sprite_dir, existing_root)?;
        let serialized = render_value_compact_arrays(&merged_root);
        let formatted = format!("{serialized}\n");

        match existing_json {
            Some(previous) => {
                if previous != formatted {
                    std::fs::write(&json_path, formatted).map_err(|error| error.to_string())?;
                    report.updated += 1;
                }
            }
            None => {
                std::fs::write(&json_path, formatted).map_err(|error| error.to_string())?;
                report.created += 1;
            }
        }
    }

    Ok(report)
}

fn scan_sprite_directories(sprites_dir: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let mut directories = BTreeMap::new();

    for entry in std::fs::read_dir(sprites_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        directories.insert(name.to_string(), path);
    }

    Ok(directories)
}

fn scan_entity_type_jsons(entity_types_dir: &Path) -> Result<HashMap<String, PathBuf>, String> {
    let mut json_files = HashMap::new();

    for entry in std::fs::read_dir(entity_types_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        json_files.insert(stem.to_string(), path);
    }

    Ok(json_files)
}

pub fn build_entity_type_json(
    entity_name: &str,
    sprite_dir: &Path,
    existing_root: Value,
) -> Result<Value, String> {
    let mut definition: EntityTypeDefinition =
        serde_json::from_value(existing_root).map_err(|error| {
            format!("Entity type file for '{entity_name}' must contain a JSON object: {error}")
        })?;
    definition.key = entity_name.to_string();

    let grouped_frames = collect_sprite_frames(entity_name, sprite_dir)?;
    let mut state_machine = definition
        .state_machine()
        .unwrap_or_else(|| StateMachineDefinition {
            initial_state: String::new(),
            ..Default::default()
        });
    let mut existing_states = std::mem::take(&mut state_machine.states);
    let reference_frame_path = grouped_frames
        .get("idle")
        .and_then(|frames| frames.first())
        .or_else(|| grouped_frames.values().find_map(|frames| frames.first()))
        .map(|frame| frame.filesystem_path.clone());

    // Determine pixel -> unit scale based on an existing numeric "height" in the
    // entity-type JSON (keep behaviour: if a numeric height exists we compute
    // scale = existing_height / image_pixel_height; otherwise fall back to the
    // hard-coded SCALE_FACTOR).
    let mut scale_pixels_to_units: f64 = 1.0 / SCALE_FACTOR;
    let mut _reference_img_w: Option<f64> = None;
    let mut _reference_img_h: Option<f64> = None;
    let existing_height_value = definition.height.map(|value| value as f64);
    if let Some(ref_frame) = &reference_frame_path {
        let image =
            image::open(ref_frame).map_err(|error| format!("{}: {error}", ref_frame.display()))?;
        let img_w = image.width() as f64;
        let img_h = image.height() as f64;
        _reference_img_w = Some(img_w);
        _reference_img_h = Some(img_h);

        if let Some(h_val) = existing_height_value {
            if img_h == 0.0 {
                return Err(format!("image has zero height: {}", ref_frame.display()));
            }
            scale_pixels_to_units = h_val / img_h;
        } else {
            scale_pixels_to_units = 1.0 / SCALE_FACTOR;
        }
    }

    let mut new_states = HashMap::new();
    let is_floor = false;

    for (state_key, frames) in grouped_frames {
        let mut state_definition = existing_states.remove(&state_key).unwrap_or_default();

        let animation_paths: Vec<String> = frames
            .iter()
            .map(|frame| frame.asset_path.clone())
            .collect();
        state_definition.animation = animation_paths;

        if should_regenerate_hitbox(&state_definition) {
            let ignore_top: u32 = if is_floor { 40 } else { 0 };
            let (_img_w_px, _img_h_px, hitbox) =
                build_hitbox_from_png(&frames[0].filesystem_path, ignore_top)?;

            // Scale hitbox coordinates by the computed pixel->unit scale and convert to JSON numbers
            // If a numeric "height" exists in the JSON the scale was set to
            // existing_height / reference_image_pixel_height earlier, otherwise we
            // use 1.0 / SCALE_FACTOR for backward compatibility.
            let mut hitbox_array: Vec<[f32; 2]> = Vec::with_capacity(hitbox.len());
            for [x, y] in hitbox.into_iter() {
                let xf = (x as f64) * scale_pixels_to_units;
                let yf = (y as f64) * scale_pixels_to_units;
                // Round to nearest whole number
                let xf_i = xf.round();
                let yf_i = yf.round();
                if xf_i < 0.0 || yf_i < 0.0 {
                    return Err(format!(
                        "rounded hitbox coordinate negative: {}, {}",
                        xf_i, yf_i
                    ));
                }
                hitbox_array.push([xf_i as f32, yf_i as f32]);
            }

            state_definition.collider_box = Some(hitbox_array);
        }

        if state_definition.animation_frame_ms == 0 {
            state_definition.animation_frame_ms = 180;
        }

        new_states.insert(state_key, state_definition);
    }

    // Preserve any pre-existing states that did not have matching sprite
    // frames. Previously these would be dropped which caused state
    // information (transitions, sounds, custom fields) to be lost on
    // regeneration. Merge leftover `existing_states` into `new_states`
    // only when the key is not already present so explicit sprite-derived
    // states win while unrelated states are preserved.
    for (k, v) in existing_states.into_iter() {
        new_states.entry(k).or_insert(v);
    }

    if state_machine.initial_state.is_empty() {
        state_machine.initial_state = if new_states.contains_key("idle") {
            "idle".to_string()
        } else {
            new_states
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "idle".to_string())
        };
    }
    state_machine.states = new_states;
    definition
        .set_state_machine(state_machine)
        .map_err(|error| error.to_string())?;

    if let Some(reference_frame_path) = reference_frame_path {
        let image = image::open(&reference_frame_path)
            .map_err(|error| format!("{}: {error}", reference_frame_path.display()))?;
        let img_w = image.width() as f64;
        let img_h = image.height() as f64;

        // If the JSON already contains a numeric "height", keep it as-is and recompute width
        if let Some(existing_height_value) = definition.height.map(|value| value as f64) {
            // compute width = aspect_ratio * existing_height_value
            if img_h == 0.0 {
                return Err(format!(
                    "image has zero height: {}",
                    reference_frame_path.display()
                ));
            }
            let width_from_height = (img_w / img_h) * existing_height_value;
            let width_i = width_from_height.round();
            if width_i < 0.0 {
                return Err(format!("rounded width negative: {}", width_i));
            }
            definition.width = Some(width_i as f32);
            // keep existing height value unchanged
        } else {
            // No existing numeric height -> compute both width and height from image
            let width_scaled = img_w / SCALE_FACTOR;
            let height_scaled = img_h / SCALE_FACTOR;
            // Round to nearest whole number
            let width_i = width_scaled.round();
            let height_i = height_scaled.round();
            if width_i < 0.0 || height_i < 0.0 {
                return Err(format!(
                    "rounded width/height negative: {}, {}",
                    width_i, height_i
                ));
            }
            definition.width = Some(width_i as f32);
            definition.height = Some(height_i as f32);
        }
    }

    serde_json::to_value(definition).map_err(|error| error.to_string())
}

fn should_regenerate_hitbox(state: &crate::core::EntityTypeStateDefinition) -> bool {
    state
        .collider_box
        .as_ref()
        .map(|hitbox| hitbox.is_empty())
        .unwrap_or(true)
}

fn render_value_compact_arrays(value: &Value) -> String {
    render_value_compact_arrays_with_indent(value, 0)
}

fn render_value_compact_arrays_with_indent(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(arr) => {
            // Render arrays compact in a single line
            let mut parts: Vec<String> = Vec::with_capacity(arr.len());
            for item in arr {
                parts.push(render_value_compact_arrays_with_indent(item, 0));
            }
            format!("[{}]", parts.join(", "))
        }
        Value::Object(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }
            let mut parts: Vec<String> = Vec::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let indent_str = "  ".repeat(indent);
            let inner_indent_str = "  ".repeat(indent + 1);
            for key in keys {
                let val = &map[key];
                let rendered = render_value_compact_arrays_with_indent(val, indent + 1);
                parts.push(format!("{}\"{}\": {}", inner_indent_str, key, rendered));
            }
            format!("{{\n{}\n{}}}", parts.join(",\n"), indent_str)
        }
    }
}

pub fn collect_sprite_frames(
    entity_name: &str,
    sprite_dir: &Path,
) -> Result<BTreeMap<String, Vec<SpriteFrame>>, String> {
    let mut grouped_frames: BTreeMap<String, Vec<SpriteFrame>> = BTreeMap::new();

    for entry in std::fs::read_dir(sprite_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some((state_key, frame_index)) = parse_sprite_file_name(entity_name, file_name) else {
            continue;
        };

        // Normalize asset path by stripping editor tags like `.bob` / `.betty`
        // from the filename before the extension so the animation lists in
        // the generated JSON do not contain character-specific suffixes.
        // If multiple files map to the same normalized path (duplicates),
        // keep only the first encountered.
        let normalized_asset_path = match Path::new(file_name).file_stem().and_then(|s| s.to_str())
        {
            Some(stem) => {
                let mut norm_stem = stem.to_string();
                for suffix in &[".bob", ".betty"] {
                    if norm_stem.ends_with(suffix) {
                        norm_stem.truncate(norm_stem.len() - suffix.len());
                        break;
                    }
                }
                if let Some(ext) = Path::new(file_name).extension().and_then(|e| e.to_str()) {
                    format!("sprites/{entity_name}/{norm_stem}.{}", ext).replace('\\', "/")
                } else {
                    format!("sprites/{entity_name}/{norm_stem}").replace('\\', "/")
                }
            }
            None => format!("sprites/{entity_name}/{file_name}").replace('\\', "/"),
        };

        let frames_vec = grouped_frames.entry(state_key.clone()).or_default();
        if !frames_vec
            .iter()
            .any(|f| f.asset_path == normalized_asset_path && f.frame_index == frame_index)
        {
            frames_vec.push(SpriteFrame {
                frame_index,
                asset_path: normalized_asset_path,
                filesystem_path: path,
            });
        }
    }

    for frames in grouped_frames.values_mut() {
        frames.sort_by(|left, right| {
            left.frame_index.cmp(&right.frame_index).then_with(|| {
                left.asset_path
                    .to_lowercase()
                    .cmp(&right.asset_path.to_lowercase())
            })
        });
    }

    Ok(grouped_frames)
}

fn parse_sprite_file_name(entity_name: &str, file_name: &str) -> Option<(String, u32)> {
    if Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())?
        .to_ascii_lowercase()
        != "png"
    {
        return None;
    }

    let stem = Path::new(file_name).file_stem()?.to_str()?;
    // Normalize out editor-character tags that may appear before the
    // numeric frame suffix (e.g. `player-idle.bob-1.png` -> `player-idle-1`).
    let mut normalized_stem = stem.to_string();
    for tag in &[".bob", ".betty", "_bob", "_betty"] {
        if normalized_stem.contains(tag) {
            normalized_stem = normalized_stem.replace(tag, "");
        }
    }
    let entity_name_lower = entity_name.to_ascii_lowercase();
    let stem_lower = normalized_stem.to_ascii_lowercase();
    let remainder = ["-", "_"].into_iter().find_map(|separator| {
        let prefix = format!("{entity_name_lower}{separator}");
        stem_lower
            .starts_with(&prefix)
            .then(|| &normalized_stem[prefix.len()..])
    })?;

    let mut parts: Vec<&str> = remainder
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return None;
    }

    let frame_index = parts
        .last()
        .and_then(|part| part.parse::<u32>().ok())
        .unwrap_or(0);
    if parts
        .last()
        .and_then(|part| part.parse::<u32>().ok())
        .is_some()
    {
        parts.pop();
    }

    if parts.is_empty() {
        return None;
    }

    let raw_state = parts.join("-").to_ascii_lowercase();
    let state_key = canonical_state_name(&raw_state);
    Some((state_key, frame_index))
}

fn canonical_state_name(raw_state: &str) -> String {
    match raw_state {
        "default" | "stand" | "idle" => "idle",
        "walk" | "run" | "move" => "moving",
        "jump" => "jumping",
        "hit" | "hurt" => "damaged",
        "die" | "dead" | "death" => "dead",
        "fight" | "fire" | "attack" | "shoot" => "range_attacking",
        other => other,
    }
    .to_string()
}

fn build_hitbox_from_png(
    png_path: &Path,
    ignore_top: u32,
) -> Result<(u32, u32, Vec<[u32; 2]>), String> {
    // New behavior: produce a single rectangular hitbox that covers the entire
    // non-transparent area (after applying ignore_top). This preserves the
    // existing special-case for 'floor' entities (ignore_top) and keeps the
    // image width/height semantics unchanged. All other polygon/simplification
    // logic is removed for this function.
    let image = image::open(png_path)
        .map_err(|error| format!("{}: {error}", png_path.display()))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    let opaque_pixels = collect_opaque_pixels(&image, ignore_top);

    if opaque_pixels.is_empty() {
        return Ok((width, height, Vec::new()));
    }

    // Compute bounding box in image coordinates (origin: top-left, y increases downwards)
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for &(x, y) in &opaque_pixels {
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }

    // Convert to bottom-left origin coordinates used elsewhere in this file.
    // For pixel rows, the bottom coordinate for the lowest opaque pixel (max_y)
    // is: image_height - max_y - 1. The top coordinate for the highest opaque
    // pixel (min_y) is: image_height - min_y.
    let img_h_i = height as i32;
    let left = min_x.max(0) as u32;
    let right = (max_x + 1).max(0) as u32; // +1 to include the full pixel column
    let bottom = (img_h_i - max_y - 1).max(0) as u32;
    let top = (img_h_i - min_y).max(0) as u32; // top is exclusive of the next row

    // Construct rectangle vertices in clockwise order (bottom-left origin):
    // bottom-left, bottom-right, top-right, top-left
    let hitbox = vec![[left, bottom], [right, bottom], [right, top], [left, top]];

    Ok((width, height, hitbox))
}

fn collect_opaque_pixels(image: &image::RgbaImage, ignore_top: u32) -> HashSet<(i32, i32)> {
    let mut opaque_pixels = HashSet::new();

    let height = image.height();
    // number of top rows to treat as transparent (ignore_top is given in pixels)
    let ignore_rows = ignore_top.min(height) as i32;

    for (x, y, pixel) in image.enumerate_pixels() {
        // image y origin is top-left in enumerate_pixels; y increases downward.
        // For floor entities we want to ignore the top `ignore_rows` rows,
        // i.e. rows with y < ignore_rows.
        if (y as i32) < ignore_rows {
            continue;
        }
        if pixel[3] > 10 {
            opaque_pixels.insert((x as i32, y as i32));
        }
    }

    opaque_pixels
}
