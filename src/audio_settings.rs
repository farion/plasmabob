use std::io;
use std::path::PathBuf;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_MUSIC_VOLUME: f32 = 0.5;
const DEFAULT_EFFECTS_VOLUME: f32 = 1.0;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AudioSettings {
    pub(crate) music_volume: f32,
    pub(crate) effects_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume: DEFAULT_MUSIC_VOLUME,
            effects_volume: DEFAULT_EFFECTS_VOLUME,
        }
    }
}

impl AudioSettings {
    pub(crate) fn load_from_disk() -> Self {
        let path = settings_file_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Self::default();
            }
            Err(error) => {
                eprintln!("Failed to read {}: {error}", path.display());
                return Self::default();
            }
        };

        let mut parsed = match serde_json::from_str::<Self>(&content) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Failed to parse {}: {error}", path.display());
                return Self::default();
            }
        };

        parsed.sanitize();
        parsed
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        let path = settings_file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        std::fs::write(path, json)
    }

    pub(crate) fn set_music_volume(&mut self, value: f32) -> bool {
        let clamped = clamp_volume(value);
        if (self.music_volume - clamped).abs() < f32::EPSILON {
            return false;
        }
        self.music_volume = clamped;
        true
    }

    pub(crate) fn set_effects_volume(&mut self, value: f32) -> bool {
        let clamped = clamp_volume(value);
        if (self.effects_volume - clamped).abs() < f32::EPSILON {
            return false;
        }
        self.effects_volume = clamped;
        true
    }

    fn sanitize(&mut self) {
        self.music_volume = clamp_volume(self.music_volume);
        self.effects_volume = clamp_volume(self.effects_volume);
    }
}

fn clamp_volume(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn settings_file_path() -> PathBuf {
    match std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(SETTINGS_FILE_NAME)))
    {
        Some(path) => path,
        None => PathBuf::from(SETTINGS_FILE_NAME),
    }
}

