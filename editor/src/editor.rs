use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use bevy::asset::AssetPlugin;
use bevy::ecs::message::MessageReader;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyCode;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use serde::Deserialize;
use crate::io::{assets_dir, load_level, next_entity_id, save_level, scan_levels, scan_worlds, LevelEntry, WorldEntry};
use crate::dashboard;
use crate::entity_types;
use crate::model::{normalize_asset_reference, EntityDefinition, EntityTypeDefinition, LevelBoundsDefinition, LevelFile};

pub(crate) fn run() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.1)))
        .init_resource::<LevelCatalog>()
        .init_resource::<EntityTypesSyncState>()
        .init_resource::<EditorUiState>()
        .init_resource::<SelectionState>()
        .init_resource::<PointerState>()
        .init_resource::<ToastState>()
        .init_resource::<SceneDirty>()
        .init_resource::<CameraFitRequested>()
        .init_resource::<ZOverlayMode>()
        .init_resource::<UndoHistory>()
        .init_resource::<UndoCaptureState>()
            .init_resource::<EntityTypeViewState>()
        .init_resource::<ClipboardEntity>()
        .init_resource::<ComponentValueMapping>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_dir().to_string_lossy().into_owned(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "PlasmaBob Level Editor".to_string(),
                        resolution: (1600, 900).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .init_state::<EditorMode>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_component_value_mapping)
        .add_systems(OnEnter(EditorMode::LevelPicker), refresh_level_catalog)
        .add_systems(EguiPrimaryContextPass, level_picker_ui.run_if(in_state(EditorMode::LevelPicker)))
        .add_systems(EguiPrimaryContextPass, entity_types::entity_type_view_ui.run_if(in_state(EditorMode::EntityTypeView)))
        .add_systems(Update, check_sync_result.run_if(in_state(EditorMode::LevelPicker)))
        .add_systems(EguiPrimaryContextPass, editing_ui.run_if(in_state(EditorMode::Editing)))
        .add_systems(
            Update,
            (
                update_pointer_world_position,
                toggle_add_menu,
                toggle_z_overlay_mode,
                toggle_keyboard_legend_overlay,
                undo_shortcut,
                copy_entity_shortcut,
                paste_entity_shortcut,
                save_shortcut,
                delete_selected_entity_shortcut,
                adjust_selected_entity_z_shortcut,
                select_entity_on_click,
                drag_selected_entity,
                move_selected_entity_with_keyboard,
                camera_controls,
                spawn_background_tiles_when_ready,
                draw_level_bounds_outline,
                draw_selection_outline,
                rebuild_scene_if_needed,
            )
                .chain()
                .run_if(in_state(EditorMode::Editing)),
        )
        .run();
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub(crate) enum EditorMode {
    #[default]
    LevelPicker,
    Editing,
    EntityTypeView,
}

#[derive(Resource, Default)]
pub(crate) struct LevelCatalog {
    pub(crate) worlds: Vec<WorldEntry>,
    pub(crate) levels: Vec<LevelEntry>,
    pub(crate) selected_world: Option<String>, // world folder name (e.g. "auralis")
    pub(crate) error: Option<String>,
}

#[derive(Resource)]
struct EditorUiState {
    show_add_menu: bool,
    show_keyboard_legend_overlay: bool,
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
struct PointerState {
    world_position: Option<Vec2>,
    over_ui: bool,
}

#[derive(Resource, Default)]
struct SelectionState {
    selected_index: Option<usize>,
    bounds_selected: bool,
    is_dragging: bool,
    drag_offset: Vec2,
}

#[derive(Resource, Default)]
struct ToastState {
    message: Option<String>,
    expires_at_seconds: f64,
}

#[derive(Resource)]
pub(crate) struct EntityTypesSyncState {
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

fn check_sync_result(
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
    sync_state: Res<EntityTypesSyncState>,
) {
    if let Ok(mut guard) = sync_state.result.lock() {
        if let Some(res) = guard.take() {
            match res {
                Ok(report) => {
                    toast.message = Some(format!(
                        "Entity types synchronized: {} created, {} updated, {} deleted",
                        report.created, report.updated, report.deleted
                    ));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                }
                Err(e) => {
                    toast.message = Some(format!("Entity types sync failed: {}", e));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 5.0;
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct SceneDirty(bool);

#[derive(Resource, Default)]
struct CameraFitRequested(bool);

#[derive(Resource, Default)]
struct ZOverlayMode {
    enabled: bool,
}

#[derive(Resource, Default)]
struct UndoHistory {
    states: VecDeque<LevelFile>,
}

#[derive(Resource, Default)]
struct UndoCaptureState {
    drag_snapshot_taken: bool,
    keyboard_move_active: bool,
}

#[derive(Resource, Default)]
struct ClipboardEntity {
    entity: Option<EntityDefinition>,
}

#[derive(Resource, Default)]
pub(crate) struct EntityTypeViewState {
    // name of the selected entity type to view in detail
    pub(crate) selected: Option<String>,
}

const UNDO_LIMIT: usize = 100;

// ---------------------------------------------------------------------------
// Component value mapping — loaded from editor/assets/component_value_mapping.json
// ---------------------------------------------------------------------------

/// Definition for a single overrideable attribute of a gameplay component.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ComponentAttributeDefinition {
    #[serde(rename = "type")]
    pub(crate) attr_type: String,
    #[serde(default)]
    pub(crate) options: Vec<String>,
}

/// Mapping of component names → attribute names → attribute definition.
/// Loaded at startup from `editor/assets/component_value_mapping.json`.
#[derive(Debug, Clone, Deserialize, Default, Resource)]
pub(crate) struct ComponentValueMapping {
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

fn setup_component_value_mapping(mut commands: Commands) {
    commands.insert_resource(load_component_value_mapping_from_disk());
}

#[derive(Resource, Clone)]
pub(crate) struct EditorDocument {
    pub(crate) level_asset_path: String,
    pub(crate) level_fs_path: PathBuf,
    pub(crate) level: LevelFile,
    pub(crate) entity_types: HashMap<String, EntityTypeDefinition>,
    pub(crate) dirty: bool,
}

#[derive(Component)]
struct EditorCamera;

#[derive(Component)]
struct SceneEntity;

#[derive(Component)]
struct RenderedLevelEntity {
    index: usize,
}

#[derive(Component)]
struct RenderedZOverlay {
    index: usize,
}

#[derive(Component)]
struct PendingBackgroundTiles {
    image: Handle<Image>,
    level_size: Vec2,
}

#[derive(Component)]
struct BackgroundTilesReady;

// Updated Z-layer presets and colors per user request:
// 150 - Foreground -> red
// 100 - Gameplay -> green
// 50  - Near Player Background -> orange
// 0   - Background -> blue
const Z_LAYER_PRESETS: [(&str, f32, [u8; 3]); 4] = [
    ("Foreground", 150.0, [255, 0, 0]),
    ("Gameplay", 100.0, [0, 255, 0]),
    ("Near Player Background", 50.0, [255, 165, 0]),
    ("Background", 0.0, [0, 0, 255]),
];

fn draw_z_layer_legend(ui: &mut egui::Ui, z: &mut f32) -> bool {
    let mut changed = false;

    ui.group(|ui| {
        ui.label("Z-Layer Legend");

        // Farbige Layer-Liste mit Preset-Buttons fuer schnelles Einsortieren.
        for (label, value, [r, g, b]) in Z_LAYER_PRESETS {
            let color = egui::Color32::from_rgb(r, g, b);
            let is_active = (*z - value).abs() < f32::EPSILON;

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 10.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);

                let text = format!("{value:>5.0} - {label}");
                if is_active {
                    ui.colored_label(egui::Color32::WHITE, format!("> {text}"));
                } else {
                    ui.label(text);
                }

                if ui.small_button("Set").clicked() {
                    *z = value;
                    changed = true;
                }
            });
        }

        ui.separator();
        ui.add_space(4.0);
        ui.label("Z-Index between 75 and 125 are game relevant and not included in the parallax effect.");
        ui.add_space(4.0);
        ui.label(format!("Current: {:.0}", *z));
    });

    changed
}

fn z_overlay_color_for_value(z: f32) -> Color {
    let mut layers: Vec<(f32, [u8; 3])> = Z_LAYER_PRESETS
        .iter()
        .map(|(_, value, rgb)| (*value, *rgb))
        .collect();
    layers.sort_by(|left, right| left.0.total_cmp(&right.0));

    if z <= layers[0].0 {
        let [r, g, b] = layers[0].1;
        return Color::srgba_u8(r, g, b, 110);
    }

    if z >= layers[layers.len() - 1].0 {
        let [r, g, b] = layers[layers.len() - 1].1;
        return Color::srgba_u8(r, g, b, 110);
    }

    for pair in layers.windows(2) {
        let (z0, [r0, g0, b0]) = pair[0];
        let (z1, [r1, g1, b1]) = pair[1];
        if z <= z1 {
            let factor = ((z - z0) / (z1 - z0)).clamp(0.0, 1.0);
            let lerp = |a: u8, b: u8| -> u8 {
                let af = a as f32;
                let bf = b as f32;
                (af + (bf - af) * factor).round() as u8
            };

            return Color::srgba_u8(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1), 110);
        }
    }

    // Fallback, sollte durch die oberen Branches nicht erreicht werden.
    Color::srgba(1.0, 1.0, 1.0, 0.43)
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, EditorCamera));
}

fn refresh_level_catalog(mut catalog: ResMut<LevelCatalog>) {
    match scan_worlds() {
        Ok(worlds) => {
            catalog.worlds = worlds;
        }
        Err(_) => {
            catalog.worlds.clear();
        }
    }

    match scan_levels() {
        Ok(levels) => {
            catalog.levels = levels;
            catalog.error = None;
        }
        Err(error) => {
            catalog.levels.clear();
            catalog.error = Some(error);
        }
    }
}

fn level_picker_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    _time: Res<Time>,
    mut catalog: ResMut<LevelCatalog>,
    mut next_state: ResMut<NextState<EditorMode>>,
    mut pointer_state: ResMut<PointerState>,
    mut selection: ResMut<SelectionState>,
    mut ui_state: ResMut<EditorUiState>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut camera_fit_requested: ResMut<CameraFitRequested>,
    mut undo_history: ResMut<UndoHistory>,
    mut undo_capture: ResMut<UndoCaptureState>,
    mut sync_state: ResMut<EntityTypesSyncState>,
    mut view_state: ResMut<EntityTypeViewState>,
    _toast: ResMut<ToastState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("PlasmaBob Level Editor");
        ui.horizontal(|_ui| {});
        ui.add_space(12.0);

        let entity_types_dir = assets_dir().join("entity_types");
        let mut entity_type_files: Vec<String> = Vec::new();
        let mut entity_type_error: Option<String> = None;
        match std::fs::read_dir(&entity_types_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            entity_type_files.push(name.to_string());
                        }
                    }
                }
                entity_type_files.sort();
            }
                Err(_) => {
                entity_type_error = Some("Entity-types directory not found: assets/entity_types".to_string());
            }
        }

        let mut open_asset_path: Option<String> = None;
        if let Some(selected) = dashboard::render_level_picker_columns(
            ui,
            &mut open_asset_path,
            &mut catalog,
            &mut sync_state,
            &entity_type_files,
            &entity_type_error,
        ) {
            view_state.selected = Some(selected.clone());
            next_state.set(EditorMode::EntityTypeView);
        }

        if let Some(asset_path) = open_asset_path {
            match load_level(&asset_path) {
                Ok(loaded) => {
                    commands.insert_resource(EditorDocument {
                        level_asset_path: loaded.level_asset_path,
                        level_fs_path: loaded.level_fs_path,
                        level: loaded.level,
                        entity_types: loaded.entity_types,
                        dirty: false,
                    });
                    selection.selected_index = None;
                    selection.is_dragging = false;
                    ui_state.show_add_menu = false;
                    undo_history.states.clear();
                    undo_capture.drag_snapshot_taken = false;
                    undo_capture.keyboard_move_active = false;
                    scene_dirty.0 = true;
                    camera_fit_requested.0 = true;
                    next_state.set(EditorMode::Editing);
                }
                Err(error) => {
                    catalog.error = Some(error);
                }
            }
        }
    });
}

fn editing_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut next_state: ResMut<NextState<EditorMode>>,
    mut pointer_state: ResMut<PointerState>,
    mut ui_state: ResMut<EditorUiState>,
    overlay_mode: Res<ZOverlayMode>,
    mut toast: ResMut<ToastState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut scene_dirty: ResMut<SceneDirty>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<SelectionState>,
    mapping: Res<ComponentValueMapping>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            let dirty_marker = if document.dirty { " *" } else { "" };
            ui.heading(format!("{}{}", document.level_asset_path, dirty_marker));
            ui.separator();
            if ui.button("Change Level").clicked() {
                next_state.set(EditorMode::LevelPicker);
            }
            ui.separator();
            if ui.button("Add Entity (A)").clicked() {
                ui_state.show_add_menu = !ui_state.show_add_menu;
            }
        });
    });

    egui::SidePanel::right("editor_sidebar")
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            // Enforce fixed inner width so all child widgets layout to fit 400px
            ui.set_min_width(400.0);
            ui.set_max_width(400.0);
            ui.heading("Selection");

            if let Some(index) = selection.selected_index {
                if let Some(entity) = document.level.entities.get(index) {
                    let id = entity.id.clone();
                    let entity_type_name = entity.entity_type.clone();
                    let current_z = entity.z_index.unwrap_or(100.0);
                    let mut x = entity.x;
                    let mut y = entity.y;
                    let mut z = current_z;
                    let mut changed = false;
                    // Clone override state so we don't hold a borrow into document below.
                    let current_overrides = entity.overrides.clone();
                    let entity_type_def = document.entity_types.get(&entity_type_name).cloned();

                    ui.label(format!("ID: {}", id));
                    ui.label(format!("Type: {}", entity_type_name));
                    ui.label(format!("Z-Index: {}", current_z));
                    ui.label("PgUp/PgDown: +/-1, with Shift: +/-10, Home: 150, End: 0");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("x:");
                        changed |= ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("y:");
                        changed |= ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("z:");
                        changed |= ui.add(egui::DragValue::new(&mut z).speed(1.0)).changed();
                    });
                    ui.add_space(6.0);

                    // --- Component Overrides ---
                    let mut override_updates: HashMap<String, serde_json::Value> = HashMap::new();
                    let mut override_removals: std::collections::HashSet<String> = Default::default();
                    let mut overrides_changed = false;

                    if let Some(et) = &entity_type_def {
                        let has_overrideable = et.components.iter()
                            .any(|comp| mapping.components.contains_key(comp.as_str()));

                        if has_overrideable {
                            ui.separator();
                            ui.label(egui::RichText::new("Overrides").strong());
                            ui.add_space(4.0);

                            for comp_name in &et.components {
                                let Some(attrs) = mapping.components.get(comp_name.as_str()) else {
                                    continue;
                                };
                                let mut sorted_attrs: Vec<(&String, &ComponentAttributeDefinition)> =
                                    attrs.iter().collect();
                                sorted_attrs.sort_by_key(|(k, _)| k.as_str());

                                for (attr_name, attr_def) in sorted_attrs {
                                    let key = format!("{comp_name}.{attr_name}");

                                    // Resolve entity-type default: prefer nested component data in
                                    // the entity-type JSON (e.g. `"effect_heal": {"heal": 30}`),
                                    // fall back to a reasonable default based on the attribute type
                                    // from the `ComponentValueMapping` when the nested value is absent.
                                    let entity_type_default: serde_json::Value = et
                                        .extra
                                        .get(comp_name.as_str())
                                        .and_then(|v| v.as_object())
                                        .and_then(|obj| obj.get(attr_name.as_str()))
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            match attr_def.attr_type.as_str() {
                                                "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                                                "enum" => serde_json::Value::String(
                                                    attr_def.options.get(0).cloned().unwrap_or_default(),
                                                ),
                                                _ => serde_json::Value::Null,
                                            }
                                        });

                                    let is_overridden = current_overrides.contains_key(&key);
                                    let mut enable_override = is_overridden;

                                    match attr_def.attr_type.as_str() {
                                        "number" => {
                                            let default_num = entity_type_default
                                                .as_f64()
                                                .unwrap_or(0.0)
                                                as f32;

                                            ui.horizontal(|ui| {
                                                let cb = ui.checkbox(
                                                    &mut enable_override,
                                                    format!("{key}:"),
                                                );
                                                if cb.changed() {
                                                    if enable_override {
                                                        // Seed with entity-type default on activation.
                                                        if let Some(n) = serde_json::Number::from_f64(default_num as f64) {
                                                            override_updates.insert(key.clone(), serde_json::Value::Number(n));
                                                        }
                                                    } else {
                                                        override_removals.insert(key.clone());
                                                    }
                                                    overrides_changed = true;
                                                }

                                                if enable_override {
                                                    let mut value = current_overrides
                                                        .get(&key)
                                                        .and_then(|v| v.as_f64())
                                                        .map(|v| v as f32)
                                                        .unwrap_or(default_num);
                                                    let before = value;
                                                    if ui.add(egui::DragValue::new(&mut value).speed(1.0)).changed()
                                                        && (value - before).abs() > f32::EPSILON
                                                    {
                                                        if let Some(n) = serde_json::Number::from_f64(value as f64) {
                                                            override_updates.insert(key, serde_json::Value::Number(n));
                                                            overrides_changed = true;
                                                        }
                                                    }
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(format!("Type default: {default_num}"))
                                                            .weak()
                                                            .italics(),
                                                    );
                                                }
                                            });
                                        }
                                        "enum" => {
                                            let default_str = entity_type_default
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string();

                                            ui.horizontal(|ui| {
                                                let cb = ui.checkbox(
                                                    &mut enable_override,
                                                    format!("{key}:"),
                                                );
                                                if cb.changed() {
                                                    if enable_override {
                                                        override_updates.insert(
                                                            key.clone(),
                                                            serde_json::Value::String(default_str.clone()),
                                                        );
                                                    } else {
                                                        override_removals.insert(key.clone());
                                                    }
                                                    overrides_changed = true;
                                                }

                                                if enable_override {
                                                    let mut current_str = current_overrides
                                                        .get(&key)
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(default_str.as_str())
                                                        .to_string();
                                                    let before_str = current_str.clone();
                                                    egui::ComboBox::from_id_salt(format!(
                                                        "override_{comp_name}_{attr_name}"
                                                    ))
                                                    .selected_text(&current_str)
                                                    .show_ui(ui, |ui| {
                                                        for option in &attr_def.options {
                                                            ui.selectable_value(
                                                                &mut current_str,
                                                                option.clone(),
                                                                option,
                                                            );
                                                        }
                                                    });
                                                    if current_str != before_str {
                                                        override_updates.insert(
                                                            key,
                                                            serde_json::Value::String(current_str),
                                                        );
                                                        overrides_changed = true;
                                                    }
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(format!("Type default: {default_str}"))
                                                            .weak()
                                                            .italics(),
                                                    );
                                                }
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            ui.add_space(4.0);
                        }
                    }

                    changed |= draw_z_layer_legend(ui, &mut z);

                    if changed || overrides_changed {
                        push_undo_snapshot(&mut undo_history, &document.level);
                        if let Some(entity) = document.level.entities.get_mut(index) {
                            entity.x = x;
                            entity.y = y;
                            entity.z_index = Some(z);
                            for k in &override_removals {
                                entity.overrides.remove(k);
                            }
                            for (k, v) in override_updates {
                                entity.overrides.insert(k, v);
                            }
                        }
                        document.dirty = true;
                        scene_dirty.0 = true;
                    }
                }
            } else if selection.bounds_selected {
                ui.label("Level background / Bounds selected");
                ui.label("Origin: (0, 0) — fixed");
                ui.add_space(8.0);

                let bounds = document.level.bounds.get_or_insert(
                    LevelBoundsDefinition { width: 1000.0, height: 1024.0 }
                );
                let mut width = bounds.width;
                let mut height = bounds.height;
                let mut bounds_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    bounds_changed |= ui.add(egui::DragValue::new(&mut width).speed(1.0).range(1.0..=50000.0)).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    bounds_changed |= ui.add(egui::DragValue::new(&mut height).speed(1.0).range(1.0..=50000.0)).changed();
                });

                if bounds_changed {
                    push_undo_snapshot(&mut undo_history, &document.level);
                    if let Some(b) = &mut document.level.bounds {
                        b.width = width.max(1.0);
                        b.height = height.max(1.0);
                    }
                    document.dirty = true;
                    scene_dirty.0 = true;
                }
            } else {
                ui.label("No selection.");
                ui.label("Click on entity or background.");
            }

        });

    if ui_state.show_keyboard_legend_overlay {
        draw_keyboard_legend_overlay(ctx, overlay_mode.enabled);
    }

    if ui_state.show_add_menu {
        let mut open = ui_state.show_add_menu;
        egui::Window::new("Add Entity-Type")
            .open(&mut open)
            .default_size([320.0, 420.0])
            .show(ctx, |ui| {
                ui.label("Choose an entity type:");
                ui.separator();

                let mut entity_type_names: Vec<_> = document.entity_types.keys().cloned().collect();
                entity_type_names.sort();

                let camera_center = camera_center_world(&camera_query, &window_query);
                let spawn_position = camera_center.unwrap_or(Vec2::ZERO);
                let mut add_requested: Option<String> = None;

                // Ensure the add-menu scroll area has its own id scope to avoid collisions with other lists
                ui.push_id("add_menu_entity_types_scroll", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for entity_type_name in entity_type_names {
                            ui.push_id(format!("addmenu_entity_type:{}", entity_type_name), |ui| {
                                if ui.button(&entity_type_name).clicked() {
                                    add_requested = Some(entity_type_name);
                                }
                            });
                        }
                    });
                });

                if let Some(entity_type_name) = add_requested {
                    let is_player = document
                        .entity_types
                        .get(&entity_type_name)
                        .map(|et| et.components.iter().any(|c| c == "player"))
                        .unwrap_or(false);

                    let player_already_exists = is_player && document.level.entities.iter().any(|e| {
                        document
                            .entity_types
                            .get(&e.entity_type)
                            .map(|et| et.components.iter().any(|c| c == "player"))
                            .unwrap_or(false)
                    });

                    if player_already_exists {
                        toast.message = Some("There can only be one player (Bob)!".to_string());
                        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    } else {
                        push_undo_snapshot(&mut undo_history, &document.level);
                        let id = next_entity_id(&entity_type_name, &document.level.entities);
                        let new_entity = EntityDefinition {
                            id,
                            entity_type: entity_type_name,
                            x: spawn_position.x,
                            y: spawn_position.y,
                            z_index: Some(100.0),
                            overrides: HashMap::new(),
                        };
                        document.level.entities.push(new_entity);
                        selection.selected_index = Some(document.level.entities.len() - 1);
                        document.dirty = true;
                        scene_dirty.0 = true;
                        ui_state.show_add_menu = false;
                    }
                }
            });
        ui_state.show_add_menu = ui_state.show_add_menu && open;
    }

    if let Some(message) = &toast.message {
        if time.elapsed_secs_f64() <= toast.expires_at_seconds {
            egui::Area::new("save_toast".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(message);
                    });
                });
        }
    }

    // Update pointer_state after constructing the UI so that clicks inside egui
    // panels (SidePanel, Windows, etc.) are considered "over_ui" for the rest
    // of this frame. This prevents clicks from "falling through" the sidebar
    // into the level viewport below.
    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();
}

fn update_pointer_world_position(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut pointer_state: ResMut<PointerState>,
) {
    let Ok(window) = window_query.single() else {
        pointer_state.world_position = None;
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        pointer_state.world_position = None;
        return;
    };

    pointer_state.world_position = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());
}

fn toggle_add_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<EditorUiState>,
    mut selection: ResMut<SelectionState>,
) {
    if keys.just_pressed(KeyCode::KeyA) {
        ui_state.show_add_menu = !ui_state.show_add_menu;
        selection.is_dragging = false;
    }
}

fn toggle_keyboard_legend_overlay(
    mut key_events: MessageReader<KeyboardInput>,
    mut ui_state: ResMut<EditorUiState>,
) {
    if logical_char_just_pressed(&mut key_events, "l") {
        ui_state.show_keyboard_legend_overlay = !ui_state.show_keyboard_legend_overlay;
    }
}

fn draw_keyboard_legend_overlay(ctx: &egui::Context, z_overlay_enabled: bool) {

    egui::Area::new("keyboard_legend_overlay".into())
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::LEFT_BOTTOM, [12.0, -12.0])
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 24, 30, 170))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                ))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_max_width(340.0);
                    ui.label(egui::RichText::new("Controls").strong());
                    ui.label("Left click: select / drag");
                    ui.label("A: Add entity");
                    ui.label("D: Remove entity");
                    ui.label("Arrows: move (Shift fast, Alt fine)");
                    ui.label("PgUp/PgDown: Z +/-1, with Shift +/-10");
                    ui.label("Home: Z=150, End: Z=0");
                    ui.label("Ctrl+C: copy entity");
                    ui.label("Ctrl+V: paste entity");
                    ui.label("Ctrl+S: save");
                    ui.label("Mouse wheel: zoom, right mouse button: pan camera");
                    let overlay_state = if z_overlay_enabled { "on" } else { "off" };
                    ui.label(format!("Z: Z-Overlay ({overlay_state})"));
                    ui.label("L: Toggle legend");
                });
        });
}

fn toggle_z_overlay_mode(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay_mode: ResMut<ZOverlayMode>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if control_pressed {
        return;
    }

    if !logical_char_just_pressed(&mut key_events, "z") {
        return;
    }

    overlay_mode.enabled = !overlay_mode.enabled;
    scene_dirty.0 = true;
    toast.message = Some(if overlay_mode.enabled {
        "Z-Overlay: on".to_string()
    } else {
        "Z-Overlay: off".to_string()
    });
    toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
}

fn logical_char_just_pressed(key_events: &mut MessageReader<KeyboardInput>, target: &str) -> bool {
    key_events.read().any(|event| {
        if event.state != ButtonState::Pressed {
            return false;
        }

        matches!(
            &event.logical_key,
            Key::Character(character) if character.eq_ignore_ascii_case(target)
        )
    })
}

fn push_undo_snapshot(history: &mut UndoHistory, level: &LevelFile) {
    if history.states.len() >= UNDO_LIMIT {
        history.states.pop_front();
    }
    history.states.push_back(level.clone());
}

fn undo_shortcut(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<UndoHistory>,
    mut capture_state: ResMut<UndoCaptureState>,
    mut document: ResMut<EditorDocument>,
    mut selection: ResMut<SelectionState>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !control_pressed || !logical_char_just_pressed(&mut key_events, "z") {
        return;
    }

    let Some(previous_level) = history.states.pop_back() else {
        toast.message = Some("Nothing to undo".to_string());
        toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
        return;
    };

    document.level = previous_level;
    document.dirty = true;
    scene_dirty.0 = true;

    selection.selected_index = None;
    selection.is_dragging = false;
    selection.drag_offset = Vec2::ZERO;
    capture_state.drag_snapshot_taken = false;
    capture_state.keyboard_move_active = false;

    toast.message = Some("Undone".to_string());
    toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
}

fn copy_entity_shortcut(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<SelectionState>,
    document: Res<EditorDocument>,
    mut clipboard: ResMut<ClipboardEntity>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !control_pressed || !logical_char_just_pressed(&mut key_events, "c") {
        return;
    }

    let Some(index) = selection.selected_index else {
        return;
    };

    let Some(entity) = document.level.entities.get(index) else {
        return;
    };

    let is_player = document
        .entity_types
        .get(&entity.entity_type)
        .map(|et| et.components.iter().any(|c| c == "player"))
        .unwrap_or(false);

    if is_player {
        toast.message = Some("Player cannot be copied!".to_string());
        toast.expires_at_seconds = time.elapsed_secs_f64() + 2.0;
    } else {
        clipboard.entity = Some(entity.clone());
        toast.message = Some(format!("Entity '{}' copied", entity.id));
        toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
    }
}

fn paste_entity_shortcut(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut selection: ResMut<SelectionState>,
    clipboard: Res<ClipboardEntity>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !control_pressed || !logical_char_just_pressed(&mut key_events, "v") {
        return;
    }

    let Some(original_entity) = &clipboard.entity else {
        toast.message = Some("Nothing to paste".to_string());
        toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
        return;
    };

    push_undo_snapshot(&mut undo_history, &document.level);

    let mut new_entity = original_entity.clone();
    new_entity.id = next_entity_id(&new_entity.entity_type, &document.level.entities);
    new_entity.x += 50.0;
    new_entity.y += 50.0;

    document.level.entities.push(new_entity);
    selection.selected_index = Some(document.level.entities.len() - 1);
    document.dirty = true;
    scene_dirty.0 = true;

    toast.message = Some("Entity inserted".to_string());
    toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
}

fn save_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut toast: ResMut<ToastState>,
    mut document: ResMut<EditorDocument>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !control_pressed || !keys.just_pressed(KeyCode::KeyS) {
        return;
    }

    match save_level(&document.level_fs_path, &document.level) {
        Ok(()) => {
            document.dirty = false;
            toast.message = Some("Saved".to_string());
            toast.expires_at_seconds = time.elapsed_secs_f64() + 2.0;
        }
        Err(error) => {
            toast.message = Some(format!("Save failed: {error}"));
            toast.expires_at_seconds = time.elapsed_secs_f64() + 4.0;
        }
    }
}

fn delete_selected_entity_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<EditorUiState>,
    mut selection: ResMut<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut scene_dirty: ResMut<SceneDirty>,
) {
    if ui_state.show_add_menu || !keys.just_pressed(KeyCode::KeyD) {
        return;
    }

    let Some(index) = selection.selected_index else {
        return;
    };
    if index >= document.level.entities.len() {
        selection.selected_index = None;
        selection.is_dragging = false;
        selection.drag_offset = Vec2::ZERO;
        return;
    }

    push_undo_snapshot(&mut undo_history, &document.level);
    document.level.entities.remove(index);
    document.dirty = true;
    scene_dirty.0 = true;
    selection.selected_index = None;
    selection.is_dragging = false;
    selection.drag_offset = Vec2::ZERO;
}

fn adjust_selected_entity_z_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<EditorUiState>,
    selection: Res<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut rendered_entities: Query<
        (&RenderedLevelEntity, &mut Transform),
        Without<RenderedZOverlay>,
    >,
    mut rendered_overlays: Query<
        (&RenderedZOverlay, &mut Transform, &mut Sprite),
        Without<RenderedLevelEntity>,
    >,
    mut scene_dirty: ResMut<SceneDirty>,
) {
    if ui_state.show_add_menu {
        return;
    }

    let Some(index) = selection.selected_index else {
        return;
    };

    let Some(current_entity) = document.level.entities.get(index) else {
        return;
    };

    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let step = if shift_pressed { 10.0 } else { 1.0 };
    let mut z = current_entity.z_index.unwrap_or(100.0);
    let mut changed = false;

    if keys.just_pressed(KeyCode::Home) {
        z = 150.0;
        changed = true;
    } else if keys.just_pressed(KeyCode::End) {
        z = 0.0;
        changed = true;
    } else {
        if keys.just_pressed(KeyCode::PageUp) {
            z += step;
            changed = true;
        }
        if keys.just_pressed(KeyCode::PageDown) {
            z -= step;
            changed = true;
        }
    }

    if !changed {
        return;
    }

    push_undo_snapshot(&mut undo_history, &document.level);
    let Some(entity) = document.level.entities.get_mut(index) else {
        return;
    };
    entity.z_index = Some(z);
    document.dirty = true;
    scene_dirty.0 = true;

    for (rendered, mut transform) in &mut rendered_entities {
        if rendered.index == index {
            transform.translation.z = z;
        }
    }

    for (rendered, mut transform, mut sprite) in &mut rendered_overlays {
        if rendered.index == index {
            transform.translation.z = z + 0.01;
            sprite.color = z_overlay_color_for_value(z);
        }
    }
}

fn select_entity_on_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    pointer_state: Res<PointerState>,
    ui_state: Res<EditorUiState>,
    document: Res<EditorDocument>,
    mut selection: ResMut<SelectionState>,
) {
    if ui_state.show_add_menu || pointer_state.over_ui || !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(pointer_world) = pointer_state.world_position else {
        return;
    };

    let hit = topmost_entity_at_position(pointer_world, &document.level, &document.entity_types);

    if let Some((index, entity_position)) = hit {
        selection.selected_index = Some(index);
        selection.bounds_selected = false;
        selection.is_dragging = true;
        selection.drag_offset = entity_position - pointer_world;
    } else if is_inside_level_bounds(pointer_world, &document.level) {
        selection.selected_index = None;
        selection.bounds_selected = true;
        selection.is_dragging = false;
    } else {
        selection.selected_index = None;
        selection.bounds_selected = false;
        selection.is_dragging = false;
    }
}

fn drag_selected_entity(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    pointer_state: Res<PointerState>,
    mut selection: ResMut<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut capture_state: ResMut<UndoCaptureState>,
    mut rendered_entities: Query<
        (&RenderedLevelEntity, &mut Transform),
        Without<RenderedZOverlay>,
    >,
    mut rendered_overlays: Query<(&RenderedZOverlay, &mut Transform), Without<RenderedLevelEntity>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) {
        selection.is_dragging = false;
        capture_state.drag_snapshot_taken = false;
        return;
    }

    if !selection.is_dragging {
        return;
    }

    let Some(pointer_world) = pointer_state.world_position else {
        return;
    };
    let Some(index) = selection.selected_index else {
        return;
    };

    let Some(current_entity) = document.level.entities.get(index) else {
        return;
    };

    let new_position = pointer_world + selection.drag_offset;

    let old_position = Vec2::new(current_entity.x, current_entity.y);
    if (new_position - old_position).length_squared() > f32::EPSILON && !capture_state.drag_snapshot_taken {
        push_undo_snapshot(&mut undo_history, &document.level);
        capture_state.drag_snapshot_taken = true;
    }

    let entity_type_name = {
        let Some(entity) = document.level.entities.get_mut(index) else {
            return;
        };

        entity.x = new_position.x;
        entity.y = new_position.y;
        entity.entity_type.clone()
    };
    document.dirty = true;

    let size = document
        .entity_types
        .get(&entity_type_name)
        .map(|entity_type| entity_type.size())
        .unwrap_or(Vec2::ZERO);
    let render_position = entity_render_center(new_position, size);

    for (rendered, mut transform) in &mut rendered_entities {
        if rendered.index == index {
            transform.translation.x = render_position.x;
            transform.translation.y = render_position.y;
        }
    }

    for (rendered, mut transform) in &mut rendered_overlays {
        if rendered.index == index {
            transform.translation.x = render_position.x;
            transform.translation.y = render_position.y;
        }
    }
}

fn move_selected_entity_with_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<EditorUiState>,
    selection: Res<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut capture_state: ResMut<UndoCaptureState>,
    mut rendered_entities: Query<
        (&RenderedLevelEntity, &mut Transform),
        Without<RenderedZOverlay>,
    >,
    mut rendered_overlays: Query<(&RenderedZOverlay, &mut Transform), Without<RenderedLevelEntity>>,
) {
    if ui_state.show_add_menu {
        capture_state.keyboard_move_active = false;
        return;
    }

    let Some(index) = selection.selected_index else {
        capture_state.keyboard_move_active = false;
        return;
    };

    let step = if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        1.0
    } else if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        10.0
    } else {
        5.0
    };

    let mut move_delta = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) {
        move_delta.x -= step;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        move_delta.x += step;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        move_delta.y += step;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        move_delta.y -= step;
    }

    if move_delta == Vec2::ZERO {
        capture_state.keyboard_move_active = false;
        return;
    }

    if !capture_state.keyboard_move_active {
        push_undo_snapshot(&mut undo_history, &document.level);
        capture_state.keyboard_move_active = true;
    }

    let (new_x, new_y) = {
        let Some(entity) = document.level.entities.get_mut(index) else {
            return;
        };

        entity.x += move_delta.x;
        entity.y += move_delta.y;
        (entity.x, entity.y)
    };
    document.dirty = true;

    let size = document
        .level
        .entities
        .get(index)
        .and_then(|entity| document.entity_types.get(&entity.entity_type))
        .map(|entity_type| entity_type.size())
        .unwrap_or(Vec2::ZERO);
    let render_position = entity_render_center(Vec2::new(new_x, new_y), size);

    for (rendered, mut transform) in &mut rendered_entities {
        if rendered.index == index {
            transform.translation.x = render_position.x;
            transform.translation.y = render_position.y;
        }
    }

    for (rendered, mut transform) in &mut rendered_overlays {
        if rendered.index == index {
            transform.translation.x = render_position.x;
            transform.translation.y = render_position.y;
        }
    }
}

fn camera_controls(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
) {
    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let current_scale = match projection.as_mut() {
        Projection::Orthographic(orthographic) => orthographic.scale,
        _ => 1.0,
    };

    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion.read().fold(Vec2::ZERO, |acc, event| acc + event.delta);
        transform.translation.x -= delta.x * current_scale;
        transform.translation.y += delta.y * current_scale;
    } else {
        mouse_motion.clear();
    }

    let zoom_delta = mouse_wheel.read().fold(0.0, |acc, event| acc + event.y);
    if zoom_delta.abs() > f32::EPSILON {
        let zoom_factor = 1.0 - (zoom_delta * 0.1);
        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scale = (orthographic.scale * zoom_factor).clamp(0.1, 20.0);
        }
    }
}

fn draw_selection_outline(
    mut gizmos: Gizmos,
    selection: Res<SelectionState>,
    document: Res<EditorDocument>,
) {
    let Some(index) = selection.selected_index else {
        return;
    };
    let Some(entity) = document.level.entities.get(index) else {
        return;
    };
    let Some(entity_type) = document.entity_types.get(&entity.entity_type) else {
        return;
    };

    let size = entity_type.size();
    let center = Vec2::new(entity.x + size.x * 0.5, entity.y + size.y * 0.5);
    gizmos.rect_2d(center, size + Vec2::splat(4.0), Color::srgb(1.0, 0.0, 0.0));
}

fn rebuild_scene_if_needed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_dirty: ResMut<SceneDirty>,
    mut camera_fit_requested: ResMut<CameraFitRequested>,
    overlay_mode: Res<ZOverlayMode>,
    document: Res<EditorDocument>,
    scene_entities: Query<Entity, With<SceneEntity>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if !scene_dirty.0 {
        return;
    }

    for entity in &scene_entities {
        commands.entity(entity).despawn();
    }

    let document_level_size = level_size(&document.level, &document.entity_types);
    spawn_background(&mut commands, &asset_server, &document.level, document_level_size);
    spawn_level_entities(
        &mut commands,
        &asset_server,
        &document.level,
        &document.entity_types,
        overlay_mode.enabled,
    );
    if camera_fit_requested.0 {
        fit_camera_to_level(&document.level, &document.entity_types, &window_query, &mut camera_query);
        camera_fit_requested.0 = false;
    }

    scene_dirty.0 = false;
}

fn spawn_background(commands: &mut Commands, asset_server: &AssetServer, level: &LevelFile, level_size: Vec2) {
    let background_path = normalize_asset_reference(&level.terrain.background);
    let image = asset_server.load(background_path);

    commands.spawn((
        SceneEntity,
        PendingBackgroundTiles {
            image,
            level_size,
        },
    ));
}

fn spawn_background_tiles_when_ready(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    pending_backgrounds: Query<(Entity, &PendingBackgroundTiles), Without<BackgroundTilesReady>>,
) {
    for (entity, pending) in &pending_backgrounds {
        let Some(image) = images.get(&pending.image) else {
            continue;
        };

        let image_width = image.texture_descriptor.size.width as f32;
        let image_height = image.texture_descriptor.size.height as f32;
        if image_width <= 0.0 || image_height <= 0.0 {
            continue;
        }

        let tile_height = pending.level_size.y.max(1.0);
        let tile_width = (image_width / image_height) * tile_height;
        let tile_count = ((pending.level_size.x / tile_width).ceil() as usize).saturating_add(1);

        for index in 0..tile_count {
            let mut sprite = Sprite::from_image(pending.image.clone());
            sprite.custom_size = Some(Vec2::new(tile_width, tile_height));
            let x = index as f32 * tile_width + tile_width * 0.5;
            let y = tile_height * 0.5;

            commands.spawn((
                SceneEntity,
                sprite,
                Transform::from_xyz(x, y, -10.0),
            ));
        }

        commands.entity(entity).insert(BackgroundTilesReady);
    }
}

fn draw_level_bounds_outline(
    mut gizmos: Gizmos,
    document: Res<EditorDocument>,
    selection: Res<SelectionState>,
) {
    let Some(bounds) = &document.level.bounds else {
        return;
    };

    let size = Vec2::new(bounds.width, bounds.height);
    let center = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);

    let color = if selection.bounds_selected {
        Color::srgb(1.0, 0.3, 0.3)
    } else {
        Color::srgb(0.2, 0.5, 1.0)
    };
    gizmos.rect_2d(center, size, color);

    if selection.bounds_selected {
        gizmos.rect_2d(center, size + Vec2::splat(6.0), Color::srgba(1.0, 0.3, 0.3, 0.4));
    }
}

fn is_inside_level_bounds(pos: Vec2, level: &LevelFile) -> bool {
    let Some(bounds) = &level.bounds else {
        return false;
    };
    pos.x >= 0.0 && pos.x <= bounds.width && pos.y >= 0.0 && pos.y <= bounds.height
}

fn spawn_level_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
    spawn_z_overlays: bool,
) {
    for (index, entity) in level.entities.iter().enumerate() {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            warn!("entity type '{}' not found", entity.entity_type);
            continue;
        };

        let size = entity_type.size();
        let z = entity.z_index.unwrap_or(100.0);
        let render_position = entity_render_center(Vec2::new(entity.x, entity.y), size);
        let transform = Transform::from_xyz(render_position.x, render_position.y, z);

        if let Some(texture_path) = entity_type.default_texture_asset_path() {
            let mut sprite = Sprite::from_image(asset_server.load(texture_path));
            sprite.custom_size = Some(size);
            commands.spawn((
                SceneEntity,
                RenderedLevelEntity { index },
                sprite,
                transform,
            ));
        } else {
            let sprite = Sprite::from_color(Color::srgba(0.4, 0.6, 1.0, 0.6), size);
            commands.spawn((
                SceneEntity,
                RenderedLevelEntity { index },
                sprite,
                transform,
            ));
        }

        if spawn_z_overlays {
            let overlay_sprite = Sprite::from_color(z_overlay_color_for_value(z), size);
            commands.spawn((
                SceneEntity,
                RenderedZOverlay { index },
                overlay_sprite,
                Transform::from_xyz(render_position.x, render_position.y, z + 0.01),
            ));
        }
    }
}

fn fit_camera_to_level(
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &mut Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let level_size = level_size(level, entity_types).max(Vec2::new(100.0, 100.0));
    transform.translation.x = level_size.x * 0.5;
    transform.translation.y = level_size.y * 0.5;

    let scale_x = level_size.x / window.width().max(1.0);
    let scale_y = level_size.y / window.height().max(1.0);
    if let Projection::Orthographic(orthographic) = projection.as_mut() {
        orthographic.scale = scale_x.max(scale_y).max(0.2) * 1.05;
    }
}

fn entity_render_center(entity_bottom_left: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(
        entity_bottom_left.x + size.x * 0.5,
        entity_bottom_left.y + size.y * 0.5,
    )
}

fn level_size(level: &LevelFile, entity_types: &HashMap<String, EntityTypeDefinition>) -> Vec2 {
    if let Some(bounds) = &level.bounds {
        return bounds.size();
    }

    let mut max_corner = Vec2::ZERO;
    for entity in &level.entities {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            continue;
        };
        let size = entity_type.size();
        max_corner.x = max_corner.x.max(entity.x + size.x);
        max_corner.y = max_corner.y.max(entity.y + size.y);
    }

    max_corner.max(Vec2::new(100.0, 100.0))
}


fn topmost_entity_at_position(
    world_position: Vec2,
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
) -> Option<(usize, Vec2)> {
    let mut best_hit: Option<(usize, f32, Vec2)> = None;

    for (index, entity) in level.entities.iter().enumerate() {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            continue;
        };
        let size = entity_type.size();
        let contains_point = world_position.x >= entity.x
            && world_position.x <= entity.x + size.x
            && world_position.y >= entity.y
            && world_position.y <= entity.y + size.y;

        if !contains_point {
            continue;
        }

        let z = entity.z_index.unwrap_or(0.0);
        match best_hit {
            Some((_, best_z, _)) if best_z > z => {}
            _ => {
                best_hit = Some((index, z, Vec2::new(entity.x, entity.y)));
            }
        }
    }

    best_hit.map(|(index, _, position)| (index, position))
}

fn camera_center_world(
    camera_query: &Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    let viewport_center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    camera.viewport_to_world_2d(camera_transform, viewport_center).ok()
}



