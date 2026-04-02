use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Number, Value};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct EntityTypeSyncReport {
    pub(crate) created: usize,
    pub(crate) updated: usize,
    pub(crate) deleted: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SpriteFrame {
    frame_index: u32,
    asset_path: String,
    filesystem_path: PathBuf,
}

const SCALE_FACTOR: f64 = 7.48;

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

pub(crate) fn save_entity_type_hitboxes(
    entity_type_name: &str,
    hitboxes_by_state: &HashMap<String, [[f32; 2]; 4]>,
) -> Result<(), String> {
    let json_path = assets_dir()
        .join("entity_types")
        .join(format!("{entity_type_name}.json"));

    let content = std::fs::read_to_string(&json_path)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;
    let mut root = serde_json::from_str::<Value>(&content)
        .map_err(|error| format!("{}: {error}", json_path.display()))?;

    let states = root
        .as_object_mut()
        .and_then(|object| object.get_mut("states"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| format!("{}: missing object 'states'", json_path.display()))?;

    for (state_key, points) in hitboxes_by_state {
        let state_object = states
            .get_mut(state_key)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("{}: missing object 'states.{state_key}'", json_path.display()))?;

        let hitbox_points = points
            .iter()
            .map(|[x, y]| {
                let rounded_x = x.round().max(0.0) as i64;
                let rounded_y = y.round().max(0.0) as i64;
                Value::Array(vec![
                    Value::Number(Number::from(rounded_x)),
                    Value::Number(Number::from(rounded_y)),
                ])
            })
            .collect::<Vec<_>>();

        state_object.insert("hitbox".to_string(), Value::Array(hitbox_points));
    }

    let serialized = render_value_compact_arrays(&root);
    std::fs::write(&json_path, format!("{serialized}\n"))
        .map_err(|error| format!("{}: {error}", json_path.display()))
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

pub(crate) fn sync_entity_types_with_sprites() -> Result<EntityTypeSyncReport, String> {
    sync_entity_types_with_paths(&assets_dir().join("sprites"), &assets_dir().join("entity_types"))
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

    for (entity_name, sprite_dir) in sprite_directories {
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

pub(crate) fn build_entity_type_json(entity_name: &str, sprite_dir: &Path, existing_root: Value) -> Result<Value, String> {
    let mut root = match existing_root {
        Value::Object(object) => object,
        _ => {
            return Err(format!(
                "Entity-Type-Datei für '{entity_name}' muss ein JSON-Objekt enthalten"
            ))
        }
    };

    let grouped_frames = collect_sprite_frames(entity_name, sprite_dir)?;
    let mut existing_states = match root.remove("states") {
        Some(Value::Object(states)) => states,
        _ => Map::new(),
    };
    let reference_frame_path = grouped_frames
        .get("default")
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
    let existing_height_value = root.get("height").and_then(|v| v.as_f64());
    if let Some(ref_frame) = &reference_frame_path {
        let image = image::open(ref_frame)
            .map_err(|error| format!("{}: {error}", ref_frame.display()))?;
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

    let mut new_states = Map::new();

    // Determine if this entity type is a floor — if so, we will treat the top 40 pixels as transparent
    let is_floor = root
        .get("component")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|it| it.as_str() == Some("floor")))
        .unwrap_or(false);

    for (state_key, frames) in grouped_frames {
        let mut state_object = match existing_states.remove(&state_key) {
            Some(Value::Object(object)) => object,
            _ => Map::new(),
        };

        let animation_paths: Vec<Value> = frames
            .iter()
            .map(|frame| Value::String(frame.asset_path.clone()))
            .collect();
        state_object.insert("animation".to_string(), Value::Array(animation_paths));

        if should_regenerate_hitbox(&state_object) {
            let ignore_top: u32 = if is_floor { 40 } else { 0 };
            let (_img_w_px, _img_h_px, hitbox) =
                build_hitbox_from_png(&frames[0].filesystem_path, ignore_top)?;

            // Scale hitbox coordinates by the computed pixel->unit scale and convert to JSON numbers
            // If a numeric "height" exists in the JSON the scale was set to
            // existing_height / reference_image_pixel_height earlier, otherwise we
            // use 1.0 / SCALE_FACTOR for backward compatibility.
            let mut hitbox_array: Vec<Value> = Vec::with_capacity(hitbox.len());
            for [x, y] in hitbox.into_iter() {
                let xf = (x as f64) * scale_pixels_to_units;
                let yf = (y as f64) * scale_pixels_to_units;
                // Round to nearest whole number
                let xf_i = xf.round();
                let yf_i = yf.round();
                if xf_i < 0.0 || yf_i < 0.0 {
                    return Err(format!("rounded hitbox coordinate negative: {}, {}", xf_i, yf_i));
                }
                let nx = Number::from(xf_i as u64);
                let ny = Number::from(yf_i as u64);
                hitbox_array.push(Value::Array(vec![Value::Number(nx), Value::Number(ny)]));
            }

            state_object.insert("hitbox".to_string(), Value::Array(hitbox_array));
        }

        if !state_object.contains_key("animation_frame_ms") {
            state_object.insert(
                "animation_frame_ms".to_string(),
                Value::Number(Number::from(180_u64)),
            );
        }

        new_states.insert(state_key, Value::Object(state_object));
    }

    root.insert("states".to_string(), Value::Object(new_states));

    if !root.contains_key("component") {
        root.insert("component".to_string(), Value::Array(Vec::new()));
    }

    if let Some(reference_frame_path) = reference_frame_path {
        let image = image::open(&reference_frame_path)
            .map_err(|error| format!("{}: {error}", reference_frame_path.display()))?;
        let img_w = image.width() as f64;
        let img_h = image.height() as f64;

        // If the JSON already contains a numeric "height", keep it as-is and recompute width
        if let Some(existing_height_value) = root.get("height").and_then(|v| v.as_f64()) {
            // compute width = aspect_ratio * existing_height_value
            if img_h == 0.0 {
                return Err(format!("image has zero height: {}", reference_frame_path.display()));
            }
            let width_from_height = (img_w / img_h) * existing_height_value;
            let width_i = width_from_height.round();
            if width_i < 0.0 {
                return Err(format!("rounded width negative: {}", width_i));
            }
            let nw = Number::from(width_i as u64);
            root.insert("width".to_string(), Value::Number(nw));
            // keep existing height value unchanged
        } else {
            // No existing numeric height -> compute both width and height from image
            let width_scaled = img_w / SCALE_FACTOR;
            let height_scaled = img_h / SCALE_FACTOR;
            // Round to nearest whole number
            let width_i = width_scaled.round();
            let height_i = height_scaled.round();
            if width_i < 0.0 || height_i < 0.0 {
                return Err(format!("rounded width/height negative: {}, {}", width_i, height_i));
            }
            let nw = Number::from(width_i as u64);
            let nh = Number::from(height_i as u64);
            root.insert("width".to_string(), Value::Number(nw));
            root.insert("height".to_string(), Value::Number(nh));
        }
    }

    Ok(Value::Object(root))
}

fn should_regenerate_hitbox(state_object: &Map<String, Value>) -> bool {
    match state_object.get("hitbox") {
        Some(Value::Array(hitbox)) => hitbox.is_empty(),
        Some(_) => true,
        None => true,
    }
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

pub(crate) fn collect_sprite_frames(entity_name: &str, sprite_dir: &Path) -> Result<BTreeMap<String, Vec<SpriteFrame>>, String> {
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

        grouped_frames
            .entry(state_key.clone())
            .or_default()
            .push(SpriteFrame {
                frame_index,
                asset_path: format!("sprites/{entity_name}/{file_name}").replace('\\', "/"),
                filesystem_path: path,
            });
    }

    for frames in grouped_frames.values_mut() {
        frames.sort_by(|left, right| {
            left.frame_index
                .cmp(&right.frame_index)
                .then_with(|| left.asset_path.to_lowercase().cmp(&right.asset_path.to_lowercase()))
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
    let entity_name_lower = entity_name.to_ascii_lowercase();
    let stem_lower = stem.to_ascii_lowercase();
    let remainder = ["-", "_"]
        .into_iter()
        .find_map(|separator| {
            let prefix = format!("{entity_name_lower}{separator}");
            stem_lower.starts_with(&prefix).then(|| &stem[prefix.len()..])
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
    if parts.last().and_then(|part| part.parse::<u32>().ok()).is_some() {
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
        "default" | "stand" | "idle" => "default",
        "walk" | "run" | "move" => "walk",
        "jump" => "jump",
        "hit" | "hurt" => "hit",
        "die" | "dead" | "death" => "die",
        "fight" | "fire" | "attack" | "shoot" => "fight",
        other => other,
    }
    .to_string()
}

fn build_hitbox_from_png(png_path: &Path, ignore_top: u32) -> Result<(u32, u32, Vec<[u32; 2]>), String> {
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

// Polygon boundary and simplification helpers were removed because
// hitboxes are now always a single rectangle covering the non-transparent
// portion of the sprite (respecting `ignore_top` for floors). The old
// functions were intentionally deleted to avoid confusion and unused code.

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use serde_json::json;
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

    fn write_png(
        root: &Path,
        relative_path: &str,
        width: u32,
        height: u32,
        opaque_pixels: &[(u32, u32)],
    ) -> PathBuf {
        let path = root.join(relative_path);
        std::fs::create_dir_all(path.parent().expect("png should have a parent"))
            .expect("png parent directory should be creatable");

        let mut image = ImageBuffer::from_pixel(width, height, Rgba([0_u8, 0_u8, 0_u8, 0_u8]));
        for &(x, y) in opaque_pixels {
            image.put_pixel(x, y, Rgba([255_u8, 255_u8, 255_u8, 255_u8]));
        }
        image.save(&path).expect("png should be writable");
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
                        "animation": ["assets/bob/bob-default.png"],
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

    #[test]
    fn parses_sprite_file_names_case_insensitive_and_with_aliases() {
        assert_eq!(
            parse_sprite_file_name("bob", "Bob-Stand.png"),
            Some(("default".to_string(), 0))
        );
        assert_eq!(
            parse_sprite_file_name("bob", "Bob-Walk-2.png"),
            Some(("walk".to_string(), 2))
        );
        assert_eq!(
            parse_sprite_file_name("betty", "betty_fire.png"),
            Some(("fight".to_string(), 0))
        );
        assert_eq!(parse_sprite_file_name("bob", "portrait.png"), None);
    }

    #[test]
    fn sync_entity_types_creates_updates_and_deletes_json_files() {
        let root = unique_temp_root();

        write_png(
            &root,
            "assets/sprites/bob/Bob-Stand.png",
            4,
            4,
            &[(1, 0), (1, 1), (1, 2), (2, 1)],
        );
        write_png(
            &root,
            "assets/sprites/bob/Bob-Walk-1.png",
            4,
            4,
            &[(0, 0), (1, 0), (2, 0)],
        );
        write_png(
            &root,
            "assets/sprites/bob/Bob-Walk-2.png",
            4,
            4,
            &[(0, 1), (1, 1), (2, 1)],
        );
        write_png(
            &root,
            "assets/sprites/bob/Bob-Fire.png",
            4,
            4,
            &[(2, 0), (2, 1), (3, 1)],
        );
        write_png(
            &root,
            "assets/sprites/betty/betty-stand.png",
            8,
            6,
            &[(1, 1), (2, 1), (2, 2), (3, 2)],
        );

        write_file(
            &root,
            "assets/entity_types/bob.json",
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "component": ["player"],
                    "health": 100,
                    "width": 99,
                    "height": 77,
                    "states": {
                        "default": {
                            "animation": ["old.png"],
                            "animation_frame_ms": 250,
                            "custom": true
                        },
                        "obsolete": {
                            "animation": ["unused.png"]
                        }
                    }
                }))
                .expect("json should serialize")
            ),
        );
        write_file(
            &root,
            "assets/entity_types/orphan.json",
            "{\n  \"component\": []\n}\n",
        );

        let report = sync_entity_types_with_paths(
            &root.join("assets/sprites"),
            &root.join("assets/entity_types"),
        )
        .expect("sync should succeed");

        assert_eq!(
            report,
            EntityTypeSyncReport {
                created: 1,
                updated: 1,
                deleted: 1,
            }
        );

        assert!(!root.join("assets/entity_types/orphan.json").exists());

        let bob_json: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("assets/entity_types/bob.json"))
                .expect("bob json should exist"),
        )
        .expect("bob json should parse");
        assert_eq!(bob_json["component"], json!(["player"]));
        assert_eq!(bob_json["health"], json!(100));
        // Existing entity-type JSON provided a "height": 77 so we must keep it and recompute width
        assert_eq!(bob_json["width"], json!(77));
        assert_eq!(bob_json["height"], json!(77));
        assert_eq!(bob_json["states"]["default"]["animation_frame_ms"], json!(250));
        assert_eq!(bob_json["states"]["default"]["custom"], json!(true));
        assert_eq!(
            bob_json["states"]["default"]["animation"],
            json!(["sprites/bob/Bob-Stand.png"])
        );
        assert_eq!(
            bob_json["states"]["walk"]["animation"],
            json!([
                "sprites/bob/Bob-Walk-1.png",
                "sprites/bob/Bob-Walk-2.png"
            ])
        );
        assert_eq!(
            bob_json["states"]["fight"]["animation"],
            json!(["sprites/bob/Bob-Fire.png"])
        );
        assert!(bob_json["states"]["obsolete"].is_null());
        let default_hitbox = bob_json["states"]["default"]["hitbox"]
            .as_array()
            .expect("hitbox should be an array");
        assert!(!default_hitbox.is_empty());
        assert!(default_hitbox.len() <= 30);

        let betty_json: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("assets/entity_types/betty.json"))
                .expect("betty json should exist"),
        )
        .expect("betty json should parse");
        assert_eq!(betty_json["component"], json!([]));
        // Scaled dimensions for betty (rounded to whole numbers)
        assert_eq!(betty_json["width"], json!(1));
        assert_eq!(betty_json["height"], json!(1));
        assert_eq!(betty_json["states"]["default"]["animation_frame_ms"], json!(180));
    }

    #[test]
    fn sync_entity_types_sets_dimensions_from_non_default_frame_when_default_is_missing() {
        let root = unique_temp_root();

        write_png(
            &root,
            "assets/sprites/slime/slime-walk-1.png",
            7,
            5,
            &[(1, 1), (2, 1), (2, 2), (3, 2)],
        );
        write_png(
            &root,
            "assets/sprites/slime/slime-hit.png",
            11,
            9,
            &[(4, 4), (5, 4), (5, 5)],
        );

        sync_entity_types_with_paths(
            &root.join("assets/sprites"),
            &root.join("assets/entity_types"),
        )
        .expect("sync should succeed");

        let slime_json: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("assets/entity_types/slime.json"))
                .expect("slime json should exist"),
        )
        .expect("slime json should parse");

        // Scaled dimensions for slime (rounded to whole numbers)
        assert_eq!(slime_json["width"], json!(1));
        assert_eq!(slime_json["height"], json!(1));
        assert_eq!(
            slime_json["states"]["hit"]["animation"],
            json!(["sprites/slime/slime-hit.png"])
        );
        assert_eq!(
            slime_json["states"]["walk"]["animation"],
            json!(["sprites/slime/slime-walk-1.png"])
        );
    }

    #[test]
    fn build_entity_type_json_preserves_existing_hitboxes_and_only_regenerates_missing_ones() {
        let root = unique_temp_root();

        write_png(
            &root,
            "assets/sprites/bob/Bob-Stand.png",
            4,
            4,
            &[(1, 0), (1, 1), (2, 1)],
        );
        write_png(
            &root,
            "assets/sprites/bob/Bob-Walk-1.png",
            4,
            4,
            &[(0, 0), (1, 0), (1, 1)],
        );

        let merged = build_entity_type_json(
            "bob",
            &root.join("assets/sprites/bob"),
            json!({
                "component": ["player"],
                "height": 8,
                "states": {
                    "default": {
                        "animation": ["old.png"],
                        "hitbox": [[10, 11], [12, 11], [12, 13], [10, 13]]
                    },
                    "walk": {
                        "animation": ["old-walk.png"],
                        "hitbox": []
                    }
                }
            }),
        )
        .expect("entity type json should build");

        assert_eq!(
            merged["states"]["default"]["hitbox"],
            json!([[10, 11], [12, 11], [12, 13], [10, 13]])
        );

        let walk_hitbox = merged["states"]["walk"]["hitbox"]
            .as_array()
            .expect("walk hitbox should be regenerated as an array");
        assert!(!walk_hitbox.is_empty());
        assert_ne!(merged["states"]["walk"]["hitbox"], json!([]));
    }
}



