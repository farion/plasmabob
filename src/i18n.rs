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
pub struct CurrentLanguage(pub String);

impl Default for CurrentLanguage {
    fn default() -> Self {
        // default to German; change to "en" if you prefer English by default
        Self("de".to_string())
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
        for (localized, mut text) in all_q.iter_mut() {
            if let Some(s) = translations.tr(&current.0, &localized.key) {
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
    for (localized, mut text) in added_q.iter_mut() {
        if let Some(s) = translations.tr(&current.0, &localized.key) {
            text.0 = s.clone();
        } else {
            text.0 = format!("{{{}}}", localized.key);
        }
    }
}


