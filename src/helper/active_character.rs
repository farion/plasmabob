use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

const FILE_NAME: &str = "active_character.json";

#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActiveCharacter {
    #[default]
    Bob,
    Betty,
}

impl ActiveCharacter {
    pub(crate) fn load_from_disk() -> Self {
        let path = file_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(error) => {
                eprintln!("Failed to read {}: {error}", path.display());
                return Self::default();
            }
        };

        match serde_json::from_str::<PersistedActiveCharacter>(&content) {
            Ok(value) => value.active_character,
            Err(error) => {
                eprintln!("Failed to parse {}: {error}", path.display());
                Self::default()
            }
        }
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        let path = file_path();
        let payload = PersistedActiveCharacter {
            active_character: *self,
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        std::fs::write(path, json)
    }

    pub(crate) fn toggle(&mut self) {
        *self = match self {
            Self::Bob => Self::Betty,
            Self::Betty => Self::Bob,
        };
    }

    pub(crate) fn menu_background_path(self) -> &'static str {
        match self {
            Self::Bob => "start_bob.jpg",
            Self::Betty => "start_betty.jpg",
        }
    }

    pub(crate) fn menu_music_path(self) -> &'static str {
        match self {
            Self::Bob => "music/start_bob.ogg",
            Self::Betty => "music/start_betty.ogg",
        }
    }

    pub(crate) fn toggle_menu_label(self) -> &'static str {
        match self {
            Self::Bob => "Transform",
            Self::Betty => "Transform",
        }
    }

    pub(crate) fn menu_logo_path(self) -> &'static str {
        match self {
            Self::Bob => "logo_bob.png",
            Self::Betty => "logo_betty.png",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PersistedActiveCharacter {
    active_character: ActiveCharacter,
}

fn file_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::ActiveCharacter;

    #[test]
    fn active_character_toggle_updates_target_label() {
        let mut character = ActiveCharacter::Bob;
        assert_eq!(character.toggle_menu_label(), "Betty");
        character.toggle();
        assert_eq!(character.toggle_menu_label(), "Bob");
    }

    #[test]
    fn active_character_theme_assets_follow_selection() {
        assert_eq!(ActiveCharacter::Bob.menu_background_path(), "start_bob.jpg");
        assert_eq!(
            ActiveCharacter::Bob.menu_music_path(),
            "music/start_bob.ogg"
        );
        assert_eq!(
            ActiveCharacter::Betty.menu_background_path(),
            "start_betty.jpg"
        );
        assert_eq!(
            ActiveCharacter::Betty.menu_music_path(),
            "music/start_betty.ogg"
        );
    }
}
