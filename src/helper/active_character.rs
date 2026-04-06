use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::io;
use crate::helper::settings::{load_field, save_field};

const FILE_NAME: &str = "settings.json";

#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActiveCharacter {
    #[default]
    Bob,
    Betty,
}

impl ActiveCharacter {
    pub(crate) fn load_from_disk() -> Self {
        load_field::<ActiveCharacter>("active_character").unwrap_or_default()
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        save_field("active_character", self)
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

/*
Old single-file format removed. settings.json is now the single source of truth.
If you need the previous standalone format, restore PersistedActiveCharacter.
*/

// file path handling moved to `helper::settings`

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
