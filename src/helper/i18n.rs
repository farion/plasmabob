use bevy::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Resource, Default)]
pub struct Translations {
    // lang -> (key -> value)
    pub map: HashMap<String, HashMap<String, String>>,
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentLanguage(pub Option<String>);

impl Default for CurrentLanguage {
    fn default() -> Self {
        // Default to automatic detection
        Self(None)
    }
}

impl CurrentLanguage {

    /// Resolve the effective language code taking into account automatic detection
    /// and the set of available translations. The function prefers an explicit
    /// selection, otherwise attempts to derive a language from environment
    /// variables and falls back to sensible defaults.
    pub fn effective(&self, translations: &Translations) -> String {
        if let Some(lang) = &self.0 {
            return lang.clone();
        }

        let mut detected = String::new();

        // If no env var provided, try platform-specific system locale as fallback
        if detected.is_empty() {
            if let Some(locale) = sys_locale::get_locale() {
                detected = locale;
            }
        }

        // Extract language code portion (e.g. "de_DE.UTF-8" -> "de")
        let code = detected
            .split(|c: char| c == '.' || c == ':' || c == '-')
            .next()
            .unwrap_or("")
            .split(|c: char| c == '_' )
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();

        if !code.is_empty() && translations.map.contains_key(&code) {
            return code;
        }

        translations.map.keys().next().cloned().unwrap_or_else(|| "en".to_string())
    }

    /// Load persisted language selection from disk (simple JSON: { "language": "en" } or null).
    pub fn load_from_disk() -> Self {
        use std::io;
        use std::path::PathBuf;

        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("language.json")))
            .unwrap_or_else(|| PathBuf::from("language.json"));

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(val) => match val.get("language") {
                        Some(serde_json::Value::String(s)) => CurrentLanguage(Some(s.clone())),
                        Some(serde_json::Value::Null) | None => CurrentLanguage(None),
                        _ => CurrentLanguage::default(),
                    },
                    Err(_) => CurrentLanguage::default(),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => CurrentLanguage::default(),
            Err(_) => CurrentLanguage::default(),
        }
    }

    /// Persist current selection to disk next to the executable.
    pub fn save_to_disk(&self) -> Result<(), std::io::Error> {
        use std::io;
        use std::path::PathBuf;

        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("language.json")))
            .unwrap_or_else(|| PathBuf::from("language.json"));

        let json = match &self.0 {
            Some(lang) => serde_json::json!({ "language": lang }),
            None => serde_json::json!({ "language": null }),
        };
        std::fs::write(path, serde_json::to_string_pretty(&json).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?)
    }
}

#[derive(Component)]
pub struct LocalizedText {
    pub key: String,
}

impl Translations {
    pub fn tr<'a>(&'a self, lang: &str, key: &str) -> Option<&'a String> {
        if let Some(lang_map) = self.map.get(lang) {
            if let Some(v) = lang_map.get(key) {
                return Some(v);
            }
        }
        // fallback to english
        if lang != "en" {
            if let Some(en_map) = self.map.get("en") {
                return en_map.get(key);
            }
        }
        None
    }
}

/// Load all JSON files under assets/i18n/*.json into the Translations resource.
pub fn load_translations(mut translations: ResMut<Translations>) {
    translations.map.clear();
    let dir = Path::new("assets/i18n");
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(text) = fs::read_to_string(&path) {
                                match serde_json::from_str::<HashMap<String, String>>(&text) {
                                    Ok(map) => {
                                        translations.map.insert(stem.to_string(), map);
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse translation file {:?}: {}", path, e);
                                    }
                                }
                            } else {
                                warn!("Failed to read translation file {:?}", path);
                            }
                        }
                    }
                }
            }
        }
    } else {
        warn!("i18n directory 'assets/i18n' not found.");
    }
}

pub fn update_localized_texts(
    translations: Res<Translations>,
    current: Res<CurrentLanguage>,
    mut params: ParamSet<(
        Query<(&LocalizedText, &mut Text)>,
        Query<(&LocalizedText, &mut Text), Added<LocalizedText>>,
    )>,
) {
    // If translations or current language changed, update ALL localized texts.
    if translations.is_changed() || current.is_changed() {
        let mut all_q = params.p0();
        let lang = current.effective(&translations);
        for (localized, mut text) in all_q.iter_mut() {
            if let Some(s) = translations.tr(&lang, &localized.key) {
                text.0 = s.clone();
            } else {
                text.0 = format!("{{{}}}", localized.key);
            }
        }
        return;
    }

    // Otherwise, update only newly added LocalizedText components so UI spawned
    // after the initial translation load is still populated.
    let mut added_q = params.p1();
    let lang = current.effective(&translations);
    for (localized, mut text) in added_q.iter_mut() {
        if let Some(s) = translations.tr(&lang, &localized.key) {
            text.0 = s.clone();
        } else {
            text.0 = format!("{{{}}}", localized.key);
        }
    }
}

/// Return a sorted list of available language codes based on loaded translations.
pub fn available_language_codes(translations: &Translations) -> Vec<String> {
    let mut v: Vec<String> = translations.map.keys().cloned().collect();
    v.sort();
    v
}



