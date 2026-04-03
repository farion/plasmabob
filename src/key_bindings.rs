use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

const FILE_NAME: &str = "keybindings.json";

/// Alle konfigurierbaren Aktionen (Debug-Funktionen sind bewusst ausgeschlossen).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum KeyAction {
    MoveLeft,
    MoveRight,
    Jump,
    Shoot,
    Fullscreen,
}

impl KeyAction {
    /// Return the i18n key for this action's human-readable label.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::MoveLeft => "action.move_left",
            Self::MoveRight => "action.move_right",
            Self::Jump => "action.jump",
            Self::Shoot => "action.shoot",
            Self::Fullscreen => "action.fullscreen",
        }
    }

    pub(crate) fn all() -> [KeyAction; 5] {
        [Self::MoveLeft, Self::MoveRight, Self::Jump, Self::Shoot, Self::Fullscreen]
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeyBindings {
    #[serde(with = "keycode_serde")]
    pub(crate) move_left: KeyCode,
    #[serde(with = "keycode_serde")]
    pub(crate) move_right: KeyCode,
    #[serde(with = "keycode_serde")]
    pub(crate) jump: KeyCode,
    #[serde(with = "keycode_serde")]
    pub(crate) shoot: KeyCode,
    #[serde(with = "keycode_serde")]
    pub(crate) fullscreen: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_left: KeyCode::ArrowLeft,
            move_right: KeyCode::ArrowRight,
            jump: KeyCode::ArrowUp,
            shoot: KeyCode::Space,
            fullscreen: KeyCode::KeyF,
        }
    }
}

impl KeyBindings {
    pub(crate) fn load_from_disk() -> Self {
        let path = file_path();
        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(e) => { eprintln!("Failed to read {}: {e}", path.display()); Self::default() }
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse {}: {e}", path.display());
                Self::default()
            }),
        }
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        let path = file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub(crate) fn get(&self, action: KeyAction) -> KeyCode {
        match action {
            KeyAction::MoveLeft => self.move_left,
            KeyAction::MoveRight => self.move_right,
            KeyAction::Jump => self.jump,
            KeyAction::Shoot => self.shoot,
            KeyAction::Fullscreen => self.fullscreen,
        }
    }

    pub(crate) fn set(&mut self, action: KeyAction, key: KeyCode) {
        match action {
            KeyAction::MoveLeft => self.move_left = key,
            KeyAction::MoveRight => self.move_right = key,
            KeyAction::Jump => self.jump = key,
            KeyAction::Shoot => self.shoot = key,
            KeyAction::Fullscreen => self.fullscreen = key,
        }
    }

    /// Prüft, ob eine Taste bereits für eine andere Aktion verwendet wird.
    pub(crate) fn is_key_already_bound(&self, key: KeyCode, exclude_action: KeyAction) -> bool {
        for action in KeyAction::all() {
            if action == exclude_action {
                continue;
            }
            if self.get(action) == key {
                return true;
            }
        }
        false
    }

    /// Gibt true zurück, wenn die Taste als Belegung erlaubt ist (kein Enter, Esc, Super).
    pub(crate) fn is_valid_binding_key(key: KeyCode) -> bool {
        !matches!(
            key,
            KeyCode::Enter
                | KeyCode::Escape
                | KeyCode::SuperLeft
                | KeyCode::SuperRight
        )
    }

    /// Gibt einen lesbaren Anzeigenamen für eine Taste zurück.
    pub(crate) fn display_name(key: KeyCode) -> &'static str {
        match key {
            KeyCode::ArrowLeft => "<-",
            KeyCode::ArrowRight => "->",
            KeyCode::ArrowUp => "Hoch",
            KeyCode::ArrowDown => "Runter",
            KeyCode::Space => "Leertaste",
            KeyCode::KeyA => "A", KeyCode::KeyB => "B", KeyCode::KeyC => "C",
            KeyCode::KeyD => "D", KeyCode::KeyE => "E", KeyCode::KeyF => "F",
            KeyCode::KeyG => "G", KeyCode::KeyH => "H", KeyCode::KeyI => "I",
            KeyCode::KeyJ => "J", KeyCode::KeyK => "K", KeyCode::KeyL => "L",
            KeyCode::KeyM => "M", KeyCode::KeyN => "N", KeyCode::KeyO => "O",
            KeyCode::KeyP => "P", KeyCode::KeyQ => "Q", KeyCode::KeyR => "R",
            KeyCode::KeyS => "S", KeyCode::KeyT => "T", KeyCode::KeyU => "U",
            KeyCode::KeyV => "V", KeyCode::KeyW => "W", KeyCode::KeyX => "X",
            KeyCode::KeyY => "Y", KeyCode::KeyZ => "Z",
            KeyCode::Digit0 => "0", KeyCode::Digit1 => "1", KeyCode::Digit2 => "2",
            KeyCode::Digit3 => "3", KeyCode::Digit4 => "4", KeyCode::Digit5 => "5",
            KeyCode::Digit6 => "6", KeyCode::Digit7 => "7", KeyCode::Digit8 => "8",
            KeyCode::Digit9 => "9",
            KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3",
            KeyCode::F4 => "F4", KeyCode::F5 => "F5", KeyCode::F6 => "F6",
            KeyCode::F7 => "F7", KeyCode::F8 => "F8", KeyCode::F9 => "F9",
            KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
            KeyCode::Tab => "Tab",
            KeyCode::Backspace => "Backspace",
            KeyCode::Delete => "Entf",
            KeyCode::Insert => "Einfg",
            KeyCode::Home => "Pos1",
            KeyCode::End => "Ende",
            KeyCode::PageUp => "Bild+",
            KeyCode::PageDown => "Bild-",
            KeyCode::Numpad0 => "Num0", KeyCode::Numpad1 => "Num1", KeyCode::Numpad2 => "Num2",
            KeyCode::Numpad3 => "Num3", KeyCode::Numpad4 => "Num4", KeyCode::Numpad5 => "Num5",
            KeyCode::Numpad6 => "Num6", KeyCode::Numpad7 => "Num7", KeyCode::Numpad8 => "Num8",
            KeyCode::Numpad9 => "Num9",
            KeyCode::NumpadAdd => "Num+", KeyCode::NumpadSubtract => "Num-",
            KeyCode::NumpadMultiply => "Num*", KeyCode::NumpadDivide => "Num/",
            KeyCode::NumpadEnter => "NumEnter", KeyCode::NumpadDecimal => "Num.",
            KeyCode::Minus => "-", KeyCode::Equal => "=",
            KeyCode::BracketLeft => "[", KeyCode::BracketRight => "]",
            KeyCode::Semicolon => ";", KeyCode::Quote => "'",
            KeyCode::Backquote => "`", KeyCode::Backslash => "\\",
            KeyCode::Comma => ",", KeyCode::Period => ".", KeyCode::Slash => "/",
            KeyCode::ControlLeft => "Ctrl", KeyCode::ControlRight => "Ctrl",
            KeyCode::ShiftLeft => "Shift", KeyCode::ShiftRight => "Shift",
            KeyCode::AltLeft => "Alt", KeyCode::AltRight => "Alt",
            _ => "?",
        }
    }
}

fn file_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(FILE_NAME))
}

pub(crate) mod keycode_serde {
    use bevy::prelude::KeyCode;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(key: &KeyCode, s: S) -> Result<S::Ok, S::Error> {
        format!("{key:?}").serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<KeyCode, D::Error> {
        let s = String::deserialize(d)?;
        str_to_keycode(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown KeyCode: {s}")))
    }

    pub(crate) fn str_to_keycode(s: &str) -> Option<KeyCode> {
        Some(match s {
            "KeyA" => KeyCode::KeyA, "KeyB" => KeyCode::KeyB, "KeyC" => KeyCode::KeyC,
            "KeyD" => KeyCode::KeyD, "KeyE" => KeyCode::KeyE, "KeyF" => KeyCode::KeyF,
            "KeyG" => KeyCode::KeyG, "KeyH" => KeyCode::KeyH, "KeyI" => KeyCode::KeyI,
            "KeyJ" => KeyCode::KeyJ, "KeyK" => KeyCode::KeyK, "KeyL" => KeyCode::KeyL,
            "KeyM" => KeyCode::KeyM, "KeyN" => KeyCode::KeyN, "KeyO" => KeyCode::KeyO,
            "KeyP" => KeyCode::KeyP, "KeyQ" => KeyCode::KeyQ, "KeyR" => KeyCode::KeyR,
            "KeyS" => KeyCode::KeyS, "KeyT" => KeyCode::KeyT, "KeyU" => KeyCode::KeyU,
            "KeyV" => KeyCode::KeyV, "KeyW" => KeyCode::KeyW, "KeyX" => KeyCode::KeyX,
            "KeyY" => KeyCode::KeyY, "KeyZ" => KeyCode::KeyZ,
            "Digit0" => KeyCode::Digit0, "Digit1" => KeyCode::Digit1, "Digit2" => KeyCode::Digit2,
            "Digit3" => KeyCode::Digit3, "Digit4" => KeyCode::Digit4, "Digit5" => KeyCode::Digit5,
            "Digit6" => KeyCode::Digit6, "Digit7" => KeyCode::Digit7, "Digit8" => KeyCode::Digit8,
            "Digit9" => KeyCode::Digit9,
            "ArrowLeft" => KeyCode::ArrowLeft, "ArrowRight" => KeyCode::ArrowRight,
            "ArrowUp" => KeyCode::ArrowUp, "ArrowDown" => KeyCode::ArrowDown,
            "Space" => KeyCode::Space, "Tab" => KeyCode::Tab,
            "Backspace" => KeyCode::Backspace, "Delete" => KeyCode::Delete,
            "Insert" => KeyCode::Insert, "Home" => KeyCode::Home, "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp, "PageDown" => KeyCode::PageDown,
            "F1" => KeyCode::F1, "F2" => KeyCode::F2, "F3" => KeyCode::F3,
            "F4" => KeyCode::F4, "F5" => KeyCode::F5, "F6" => KeyCode::F6,
            "F7" => KeyCode::F7, "F8" => KeyCode::F8, "F9" => KeyCode::F9,
            "F10" => KeyCode::F10, "F11" => KeyCode::F11, "F12" => KeyCode::F12,
            "Numpad0" => KeyCode::Numpad0, "Numpad1" => KeyCode::Numpad1,
            "Numpad2" => KeyCode::Numpad2, "Numpad3" => KeyCode::Numpad3,
            "Numpad4" => KeyCode::Numpad4, "Numpad5" => KeyCode::Numpad5,
            "Numpad6" => KeyCode::Numpad6, "Numpad7" => KeyCode::Numpad7,
            "Numpad8" => KeyCode::Numpad8, "Numpad9" => KeyCode::Numpad9,
            "NumpadAdd" => KeyCode::NumpadAdd, "NumpadSubtract" => KeyCode::NumpadSubtract,
            "NumpadMultiply" => KeyCode::NumpadMultiply, "NumpadDivide" => KeyCode::NumpadDivide,
            "NumpadEnter" => KeyCode::NumpadEnter, "NumpadDecimal" => KeyCode::NumpadDecimal,
            "Minus" => KeyCode::Minus, "Equal" => KeyCode::Equal,
            "BracketLeft" => KeyCode::BracketLeft, "BracketRight" => KeyCode::BracketRight,
            "Semicolon" => KeyCode::Semicolon, "Quote" => KeyCode::Quote,
            "Backquote" => KeyCode::Backquote, "Backslash" => KeyCode::Backslash,
            "Comma" => KeyCode::Comma, "Period" => KeyCode::Period, "Slash" => KeyCode::Slash,
            "ControlLeft" => KeyCode::ControlLeft, "ControlRight" => KeyCode::ControlRight,
            "ShiftLeft" => KeyCode::ShiftLeft, "ShiftRight" => KeyCode::ShiftRight,
            "AltLeft" => KeyCode::AltLeft, "AltRight" => KeyCode::AltRight,
            _ => return None,
        })
    }
}

