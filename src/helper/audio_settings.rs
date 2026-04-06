use std::io;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use crate::helper::settings::{load_field, save_field};
const DEFAULT_MUSIC_VOLUME: f32 = 0.2;
const DEFAULT_SOUNDS_VOLUME: f32 = 0.5;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AudioSettings {
    pub(crate) music_volume: f32,
    #[serde(default = "default_sounds_volume")]
    pub(crate) sounds_volume: f32,
}

fn default_sounds_volume() -> f32 {
    DEFAULT_SOUNDS_VOLUME
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume: DEFAULT_MUSIC_VOLUME,
            sounds_volume: DEFAULT_SOUNDS_VOLUME,
        }
    }
}

impl AudioSettings {
    pub(crate) fn load_from_disk() -> Self {
        let mut parsed = load_field::<AudioSettings>("audio_settings").unwrap_or_default();
        parsed.sanitize();
        parsed
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        save_field("audio_settings", self)
    }

    pub(crate) fn set_music_volume(&mut self, value: f32) -> bool {
        let clamped = clamp_volume(value);
        if (self.music_volume - clamped).abs() < f32::EPSILON {
            return false;
        }
        self.music_volume = clamped;
        true
    }

    pub(crate) fn set_sounds_volume(&mut self, value: f32) -> bool {
        let clamped = clamp_volume(value);
        if (self.sounds_volume - clamped).abs() < f32::EPSILON {
            return false;
        }
        self.sounds_volume = clamped;
        true
    }

    fn sanitize(&mut self) {
        self.music_volume = clamp_volume(self.music_volume);
        self.sounds_volume = clamp_volume(self.sounds_volume);
    }
}

fn clamp_volume(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}
