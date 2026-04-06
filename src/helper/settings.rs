use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use std::io;
use std::path::PathBuf;

pub(crate) const SETTINGS_FILE_NAME: &str = "settings.json";

fn settings_file_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(SETTINGS_FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(SETTINGS_FILE_NAME))
}

/// Load the entire settings.json as a JSON object map. If the file is missing
/// or invalid, an empty map is returned.
pub(crate) fn load_settings() -> Map<String, Value> {
    let path = settings_file_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(Value::Object(map)) => map,
            Ok(_) => {
                eprintln!("{} does not contain a JSON object, treating as empty settings", path.display());
                Map::new()
            }
            Err(err) => {
                eprintln!("Failed to parse {}: {}", path.display(), err);
                Map::new()
            }
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => Map::new(),
        Err(e) => {
            eprintln!("Failed to read {}: {}", path.display(), e);
            Map::new()
        }
    }
}

/// Save the provided settings map to disk (pretty-printed).
pub(crate) fn save_settings(map: &Map<String, Value>) -> io::Result<()> {
    let path = settings_file_path();
    let json = serde_json::to_string_pretty(&Value::Object(map.clone()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

/// Get and deserialize a field from a settings map. Returns None if the key
/// does not exist or if deserialization failed.
pub(crate) fn get_field<T: DeserializeOwned>(map: &Map<String, Value>, key: &str) -> Option<T> {
    map.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Insert a serializable value into the settings map.
pub(crate) fn set_field<S: Serialize>(map: &mut Map<String, Value>, key: &str, value: &S) -> Result<(), serde_json::Error> {
    let val = serde_json::to_value(value)?;
    map.insert(key.to_string(), val);
    Ok(())
}

/// Convenience: load a single field from disk. Returns None if the key is
/// missing or parsing failed.
pub(crate) fn load_field<T: DeserializeOwned>(key: &str) -> Option<T> {
    let map = load_settings();
    get_field(&map, key)
}

/// Convenience: save a single field to disk (merges with existing settings).
pub(crate) fn save_field<S: Serialize>(key: &str, value: &S) -> io::Result<()> {
    let mut map = load_settings();
    match set_field(&mut map, key, value) {
        Ok(()) => save_settings(&map),
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
}

