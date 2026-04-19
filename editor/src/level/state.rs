use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::io::{LevelEntry, WorldEntry};
use crate::model::{EntityDefinition, EntityTypeDefinition, LevelFile};

#[derive(Resource, Default)]
pub struct LevelCatalog {
    pub(crate) worlds: Vec<WorldEntry>,
    pub(crate) levels: Vec<LevelEntry>,
    pub(crate) selected_world: Option<String>, // world folder name (e.g. "auralis")
    pub(crate) error: Option<String>,
}

#[derive(Resource)]
pub struct EditorUiState {
    pub(crate) show_add_menu: bool,
    pub(crate) show_keyboard_legend_overlay: bool,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_add_menu: false,
            show_keyboard_legend_overlay: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct PointerState {
    pub(crate) world_position: Option<Vec2>,
    pub(crate) over_ui: bool,
}

#[derive(Resource, Default)]
pub struct SelectionState {
    pub(crate) selected_index: Option<usize>,
    pub(crate) bounds_selected: bool,
    pub(crate) is_dragging: bool,
    pub(crate) drag_offset: Vec2,
}

#[derive(Resource, Default)]
pub struct ToastState {
    pub(crate) message: Option<String>,
    pub(crate) expires_at_seconds: f64,
}

#[derive(Resource)]
pub struct EntityTypesSyncState {
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) result: Arc<Mutex<Option<Result<crate::io::EntityTypeSyncReport, String>>>>,
}

impl Default for EntityTypesSyncState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            result: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Resource, Default)]
pub struct SceneDirty(pub bool);

#[derive(Resource, Default)]
pub struct CameraFitRequested(pub bool);

#[derive(Resource, Default)]
pub struct ZOverlayMode {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct HitboxOverlayState {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct SnapState {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct UndoHistory {
    pub states: VecDeque<LevelFile>,
}

#[derive(Resource, Default)]
pub struct UndoCaptureState {
    pub drag_snapshot_taken: bool,
    pub keyboard_move_active: bool,
}

#[derive(Resource, Default)]
pub struct ClipboardEntity {
    pub entity: Option<EntityDefinition>,
}

#[derive(Resource, Default)]
pub struct EntityTypeViewState {
    // name of the selected entity type to view in detail
    pub selected: Option<String>,
}

/// Definition for a single overrideable attribute of a gameplay component.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentAttributeDefinition {
    #[serde(rename = "type")]
    pub(crate) attr_type: String,
    #[serde(default)]
    pub(crate) options: Vec<String>,
}

/// Mapping of component names → attribute names → attribute definition.
/// Loaded at startup from `editor/assets/component_value_mapping.json`.
#[derive(Debug, Clone, Deserialize, Default, Resource)]
pub struct ComponentValueMapping {
    #[serde(default)]
    pub(crate) components: HashMap<String, HashMap<String, ComponentAttributeDefinition>>,
}

fn load_component_value_mapping_from_disk() -> ComponentValueMapping {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/component_value_mapping.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ComponentValueMapping::default(),
    }
}

pub fn setup_component_value_mapping(mut commands: Commands) {
    commands.insert_resource(load_component_value_mapping_from_disk());
}

/// Push current level snapshot into undo history with a fixed maximum depth.
pub(crate) fn push_undo_snapshot(history: &mut UndoHistory, level: &LevelFile) {
    if history.states.len() >= 100 {
        history.states.pop_front();
    }
    history.states.push_back(level.clone());
}

#[derive(Resource, Clone)]
pub struct EditorDocument {
    pub level_asset_path: String,
    pub level_fs_path: PathBuf,
    pub level: LevelFile,
    pub entity_types: HashMap<String, EntityTypeDefinition>,
    pub dirty: bool,
}
