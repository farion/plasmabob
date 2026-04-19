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
use egui_phosphor_icons::add_fonts;
pub(crate) mod table_ui;
use crate::io::{assets_dir, load_level, next_entity_id, save_level, scan_levels, scan_worlds};
use crate::dashboard;
use crate::entity_type;
// Re-export selected level state types so existing modules referencing
// `crate::editor::...` continue to work during the migration.
pub use crate::level::state::{
    LevelCatalog,
    EntityTypesSyncState,
    EditorUiState,
    PointerState,
    SelectionState,
    ToastState,
    SceneDirty,
    CameraFitRequested,
    ZOverlayMode,
    HitboxOverlayState,
    // SnapState is defined in both bevy::prelude and our level::state; avoid
    // importing the level::state SnapState into this namespace to prevent
    // duplicate type definitions. We'll refer to the level::SnapState fully
    // qualified where necessary.
    UndoHistory,
    UndoCaptureState,
    ClipboardEntity,
    EntityTypeViewState,
    ComponentValueMapping,
    EditorDocument,
    ComponentAttributeDefinition,
};
use crate::model::{normalize_asset_reference, ComponentsDefinition, EntityDefinition, EntityTypeDefinition, LevelBoundsDefinition, LevelFile};
use crate::level::push_undo_snapshot;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActiveCharacter {
    Bob,
    Betty,
}

impl Default for ActiveCharacter {
    fn default() -> Self {
        ActiveCharacter::Bob
    }
}

impl ActiveCharacter {
    fn load_from_disk() -> Self {
        let path = Self::settings_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(s) = value.get("active_character").and_then(|v| v.as_str()) {
                        match s.to_ascii_lowercase().as_str() {
                            "betty" => ActiveCharacter::Betty,
                            _ => ActiveCharacter::Bob,
                        }
                    } else {
                        ActiveCharacter::default()
                    }
                } else {
                    ActiveCharacter::default()
                }
            }
            Err(_) => ActiveCharacter::default(),
        }
    }

    pub(crate) fn save_to_disk(&self) -> Result<(), io::Error> {
        let path = Self::settings_path();
        let obj = serde_json::json!({"active_character": match self { ActiveCharacter::Bob => "bob", ActiveCharacter::Betty => "betty" }});
        std::fs::write(path, serde_json::to_string_pretty(&obj).unwrap_or_else(|_| String::new()))
    }

    fn settings_path() -> PathBuf {
        // Prefer to place editor settings next to the running binary (target folder).
        // Fallback to workspace root if the binary path cannot be determined.
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return parent.join("editor_settings.json");
            }
        }
        crate::io::workspace_root().join("editor_settings.json")
    }

    pub(crate) fn toggle(&mut self) {
        *self = match self {
            ActiveCharacter::Bob => ActiveCharacter::Betty,
            ActiveCharacter::Betty => ActiveCharacter::Bob,
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

pub(crate) fn flatten_entity_components(
    components: Option<&ComponentsDefinition>,
) -> HashMap<String, serde_json::Value> {
    let mut flat = HashMap::new();
    let Some(components) = components else {
        return flat;
    };

    let Some(components) = serde_json::to_value(components)
        .ok()
        .and_then(|v| v.as_object().cloned())
    else {
        return flat;
    };

    for (component_name, component_value) in components {
        let Some(component_object) = component_value.as_object() else {
            continue;
        };
        for (attribute_name, attribute_value) in component_object {
            flat.insert(
                format!("{component_name}.{attribute_name}"),
                attribute_value.clone(),
            );
        }
    }

    flat
}

pub(crate) fn apply_flat_component_updates(
    entity: &mut EntityDefinition,
    removals: &std::collections::HashSet<String>,
    updates: HashMap<String, serde_json::Value>,
) {
    for key in removals {
        let Some((component, attribute)) = key.split_once('.') else {
            continue;
        };
        entity.remove_component_attribute(component, attribute);
    }

    for (key, value) in updates {
        let Some((component, attribute)) = key.split_once('.') else {
            continue;
        };
        entity.set_component_attribute_value(component, attribute, value);
    }
}

pub(crate) fn is_player_entity_type(entity_type: &EntityTypeDefinition) -> bool {
    entity_type
        .category_tag
        .as_deref()
        .map(|tag| tag.eq_ignore_ascii_case("player"))
        .unwrap_or(false)
        || entity_type.has_component("controlled_movement")
}

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
    .init_resource::<HitboxOverlayState>()
    .init_resource::<crate::level::state::SnapState>()
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
        // Table UI resources
        .init_resource::<table_ui::ColumnWidths>()
        .insert_resource(ActiveCharacter::load_from_disk())
        .init_state::<EditorMode>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, crate::level::state::setup_component_value_mapping)
        .add_systems(EguiPrimaryContextPass, setup_phosphor_fonts)
        .add_systems(OnEnter(EditorMode::LevelPicker), refresh_level_catalog)
        .add_systems(EguiPrimaryContextPass, crate::level::level_picker_ui.run_if(in_state(EditorMode::LevelPicker)))
        .add_systems(EguiPrimaryContextPass, entity_type::entity_type_view_ui.run_if(in_state(EditorMode::EntityTypeView)))
        .add_systems(Update, check_sync_result.run_if(in_state(EditorMode::LevelPicker)))
        .add_systems(EguiPrimaryContextPass, crate::level::editing_ui.run_if(in_state(EditorMode::Editing)))
        .add_systems(Update, crate::level::toggle_hitbox_overlay.run_if(in_state(EditorMode::Editing)))
        .add_systems(Update, draw_hitbox_outlines.run_if(in_state(EditorMode::Editing)))
        .add_systems(
            Update,
                (
                    crate::level::update_pointer_world_position,
                    crate::level::toggle_add_menu,
                    crate::level::toggle_z_overlay_mode,
                    crate::level::toggle_snap,
                    crate::level::toggle_keyboard_legend_overlay,
                    crate::level::undo_shortcut,
                    crate::level::copy_entity_shortcut,
                    crate::level::paste_entity_shortcut,
                    crate::level::save_shortcut,
                    crate::level::delete_selected_entity_shortcut,
                    crate::level::adjust_selected_entity_z_shortcut,
                    crate::level::select_entity_on_click,
                    crate::level::drag_selected_entity,
                    crate::level::move_selected_entity_with_keyboard,
                    crate::level::camera_controls,
                    crate::level::scene::spawn_background_tiles_when_ready,
                    draw_level_bounds_outline,
                    draw_selection_outline,
                    crate::level::scene::rebuild_scene_if_needed,
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

#[derive(Component)]
pub(crate) struct EditorCamera;

#[derive(Component)]
pub(crate) struct SceneEntity;

#[derive(Component)]
pub(crate) struct RenderedLevelEntity {
    pub(crate) index: usize,
}

#[derive(Component)]
pub(crate) struct RenderedZOverlay {
    pub(crate) index: usize,
}

#[derive(Component)]
pub(crate) struct PendingBackgroundTiles {
    pub(crate) image: Handle<Image>,
    pub(crate) level_size: Vec2,
}

#[derive(Component)]
pub(crate) struct BackgroundTilesReady;

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

// draw_z_layer_legend wrapper moved to editor/src/level/ui.rs which currently
// calls into editor::draw_z_layer_legend. The authoritative UI implementation
// now lives in editor/src/level/ui.rs.

pub(crate) fn z_overlay_color_for_value(z: f32) -> Color {
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

fn setup_phosphor_fonts(
    mut contexts: EguiContexts,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        let mut fonts = egui::FontDefinitions::default();
        add_fonts(&mut fonts);
        ctx.set_fonts(fonts);
    }
}

// Distance in world units within which edges/corners will snap together.
const SNAP_THRESHOLD: f32 = 40.0;

// Note: SnapState is provided by `crate::level::state::SnapState`.
// The local duplicate was removed to avoid conflicting type definitions.

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

// level_picker_ui moved to editor/src/level/ui.rs

fn editing_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut next_state: ResMut<NextState<EditorMode>>,
    mut pointer_state: ResMut<PointerState>,
    mut ui_state: ResMut<EditorUiState>,
    overlay_mode: Res<ZOverlayMode>,
    hitbox_overlay: Res<HitboxOverlayState>,
    mut toast: ResMut<ToastState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut scene_dirty: ResMut<SceneDirty>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<SelectionState>,
    mapping: Res<ComponentValueMapping>,
    mut show_close_confirm: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();

    egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let dirty_marker = if document.dirty { " *" } else { "" };
            ui.heading(format!("{}{}", document.level_asset_path, dirty_marker));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui_phosphor_icons::icons::X).clicked() {
                    if document.dirty {
                        *show_close_confirm = true;
                    } else {
                        next_state.set(EditorMode::LevelPicker);
                    }
                }
                ui.add_space(8.0);
                if ui.button(egui_phosphor_icons::icons::PLUS).clicked() {
                    ui_state.show_add_menu = !ui_state.show_add_menu;
                }
            });
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
                    let current_overrides = flatten_entity_components(entity.components.as_ref());
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
                        let component_names = et.component_names();
                        let has_overrideable = component_names.iter()
                            .any(|comp| mapping.components.contains_key(comp.as_str()));

                        if has_overrideable {
                            ui.separator();
                            ui.label(egui::RichText::new("Overrides").strong());
                            ui.add_space(4.0);

                            for comp_name in &component_names {
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
                                        .component_attribute_value(comp_name.as_str(), attr_name.as_str())
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

                    changed |= crate::level::draw_z_layer_legend(ui, &mut z);

                    if changed || overrides_changed {
                        push_undo_snapshot(&mut undo_history, &document.level);
                        if let Some(entity) = document.level.entities.get_mut(index) {
                            entity.x = x;
                            entity.y = y;
                            entity.z_index = Some(z);
                            apply_flat_component_updates(entity, &override_removals, override_updates);
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
        crate::level::draw_keyboard_legend_overlay(ctx, overlay_mode.enabled, hitbox_overlay.enabled);
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
                    egui::ScrollArea::vertical()
                        .id_salt("editor_add_menu_entity_types_scroll")
                        .show(ui, |ui| {
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
                        .map(is_player_entity_type)
                        .unwrap_or(false);

                    let player_already_exists = is_player && document.level.entities.iter().any(|e| {
                        document
                            .entity_types
                            .get(&e.entity_type)
                            .map(is_player_entity_type)
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
                            name: None,
                            layer: None,
                            components: None,
                            extra: HashMap::new(),
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
                        // Make the toast wider so most messages fit on one line.
                        // We set a reasonable minimum width while allowing it to grow.
                        ui.set_min_width(420.0);
                        ui.set_max_width(900.0);
                        ui.label(message);
                    });
                });
        }
    }

    // Confirm close dialog for the level editor when requested
    if *show_close_confirm {
        egui::Window::new("Confirm Close")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("There are unsaved changes.");
                ui.label("Save before closing?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save and Close").clicked() {
                        match save_level(&document.level_fs_path, &document.level) {
                            Ok(()) => {
                                document.dirty = false;
                                toast.message = Some("Saved".to_string());
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 2.0;
                                *show_close_confirm = false;
                                next_state.set(EditorMode::LevelPicker);
                            }
                            Err(error) => {
                                toast.message = Some(format!("Save failed: {}", error));
                                toast.expires_at_seconds = time.elapsed_secs_f64() + 4.0;
                            }
                        }
                    }

                    if ui.button("Discard and Close").clicked() {
                        // Drop in-memory dirty flag and close
                        document.dirty = false;
                        *show_close_confirm = false;
                        next_state.set(EditorMode::LevelPicker);
                    }

                    if ui.button("Cancel").clicked() {
                        *show_close_confirm = false;
                    }
                });
            });
    }

    // Update pointer_state after constructing the UI so that clicks inside egui
    // panels (SidePanel, Windows, etc.) are considered "over_ui" for the rest
    // of this frame. This prevents clicks from "falling through" the sidebar
    // into the level viewport below.
    pointer_state.over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();
}

// Moved to editor/src/level/input.rs

// moved to editor/src/level/input.rs

// moved to editor/src/level/input.rs

// draw_keyboard_legend_overlay moved to editor/src/level/ui.rs

// moved to editor/src/level/input.rs

// moved to editor/src/level/input.rs

// moved to editor/src/level/input.rs

// moved to editor/src/level/input.rs

// push_undo_snapshot moved to editor/src/level/input.rs (used by level/ui.rs)

// Input systems moved to editor/src/level/input.rs

fn draw_selection_outline(
    mut gizmos: Gizmos,
    selection: Res<SelectionState>,
    document: Res<EditorDocument>,
    time: Res<Time>,
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

    // Animated dotted outline ("ant trail")
    // Parameters (tweakable): dot size, spacing and speed
    let dot_size = 1.0_f32;
    let spacing = 4.0_f32; // distance between dot centers along edges
    let speed = 20.0_f64; // pixels per second along the outline

    // Perimeter points are computed in clockwise order starting at left-bottom
    let left = entity.x;
    let bottom = entity.y;
    let right = entity.x + size.x;
    let top = entity.y + size.y;

    // Total perimeter length
    let perim = 2.0 * (size.x + size.y);

    // Compute animation offset along perimeter in [0, spacing)
    let t = time.elapsed_secs_f64();
    let offset = ((t * speed) % (spacing as f64)) as f32;

    // Helper to convert distance along perimeter to world position
    let point_at = |mut dist: f32| -> Vec2 {
        dist = dist % perim;
        if dist <= size.x {
            // bottom edge: left -> right
            Vec2::new(left + dist, bottom)
        } else if dist <= size.x + size.y {
            // right edge: bottom -> top
            Vec2::new(right, bottom + (dist - size.x))
        } else if dist <= size.x + size.y + size.x {
            // top edge: right -> left
            Vec2::new(right - (dist - (size.x + size.y)), top)
        } else {
            // left edge: top -> bottom
            Vec2::new(left, top - (dist - (size.x + size.y + size.x)))
        }
    };

    // Place dots spaced by `spacing` along the perimeter, starting at offset
    let mut dist = offset;
    while dist < perim {
        let p = point_at(dist);
        // center the dot (small square) on the edge position
        gizmos.rect_2d(p, Vec2::splat(dot_size), Color::srgb(0.2, 0.5, 1.0));
        dist += spacing;
    }
}

fn draw_hitbox_outlines(
    mut gizmos: Gizmos,
    document: Res<EditorDocument>,
    hitbox_overlay: Res<HitboxOverlayState>,
) {
    if !hitbox_overlay.enabled {
        return;
    }

    for entity in &document.level.entities {
        let Some(entity_type) = document.entity_types.get(&entity.entity_type) else {
            continue;
        };

        let size = entity_type.size();

        // Hitbox outline: red (default = full sprite bounds).
        let mut min_x = 0.0_f32;
        let mut max_x = size.x;
        let mut min_y = 0.0_f32;
        let mut max_y = size.y;

        if let Some(sm) = entity_type.state_machine() {
            let state_key = if !sm.initial_state.is_empty() {
                sm.initial_state.clone()
            } else {
                sm.states.keys().next().cloned().unwrap_or_default()
            };

            if let Some(state_def) = sm.states.get(&state_key) {
                if let Some(box_pts) = state_def.collider_box.as_ref() {
                    if box_pts.len() >= 2 {
                        min_x = box_pts.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                        max_x = box_pts.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
                        min_y = box_pts.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                        max_y = box_pts.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                    }
                }
            }
        }

        let hit_w = (max_x - min_x).max(0.0);
        let hit_h = (max_y - min_y).max(0.0);
        let hit_center = Vec2::new(
            entity.x + min_x + hit_w * 0.5,
            entity.y + min_y + hit_h * 0.5,
        );
        gizmos.rect_2d(hit_center, Vec2::new(hit_w, hit_h), Color::srgb(1.0, 0.0, 0.0));
    }
}

// Scene rebuild moved into editor::level::scene module.

// Character-aware asset resolution is delegated to `src/helper/asset_io.rs`.
// The helper provides `resolve_character_asset_path(asset_server, path, active)`
// which uses the same algorithm as the game runtime and checks the underlying
// AssetServer source for existence.

pub(crate) fn resolve_character_asset_path(
    asset_server: &AssetServer,
    asset_path: &str,
    active: ActiveCharacter,
) -> Result<String, std::io::Error> {
    let _ = asset_server; // parameter currently unused - keep for API parity with the game helper
    // Prefer the exact normalized path when present on disk under the editor's
    // assets/ directory. If the exact file is missing, try inserting the
    // character suffix before the extension and check that path as well.
    let normalized = normalize_asset_reference(asset_path);

    let fs_exact = assets_dir().join(&normalized);
    if std::fs::metadata(&fs_exact).is_ok() {
        warn!("using exact asset: {}", normalized);
        return Ok(normalized);
    }

    // Try suffixed variant only when the original does not exist.
    if let Some(pos) = normalized.rfind('.') {
        let (before_ext, ext) = normalized.split_at(pos); // ext includes the dot
        // If the name already contains a character suffix, return normalized
        if before_ext.ends_with(".bob") || before_ext.ends_with(".betty") {
            return Ok(normalized);
        }
        let suf = match active {
            ActiveCharacter::Bob => "bob",
            ActiveCharacter::Betty => "betty",
        };
        let suffixed = format!("{}.{suf}{}", before_ext, ext);
        let fs_suff = assets_dir().join(&suffixed);
        if std::fs::metadata(&fs_suff).is_ok() {
            warn!("using character-suffixed asset: {}", suffixed);
            return Ok(suffixed);
        }
    }

    warn!("asset not found, returning normalized path: {}", normalized);
    Ok(normalized)
}

// append_character_suffix_before_extension removed: the resolver uses a
// simple inline suffix construction (match on `active`) and filesystem checks
// against `assets/` to determine the correct asset path. Keeping the helper
// here previously duplicated logic from the game's helper and caused unused
// function warnings in the editor crate.


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

pub(crate) fn is_inside_level_bounds(pos: Vec2, level: &LevelFile) -> bool {
    let Some(bounds) = &level.bounds else {
        return false;
    };
    pos.x >= 0.0 && pos.x <= bounds.width && pos.y >= 0.0 && pos.y <= bounds.height
}



pub(crate) fn entity_render_center(entity_bottom_left: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(
        entity_bottom_left.x + size.x * 0.5,
        entity_bottom_left.y + size.y * 0.5,
    )
}

/// Apply snapping to the entity at `index` within `document` when enabled.
/// Snaps edges and corners to nearby entities when within `SNAP_THRESHOLD`.
pub(crate) fn apply_snapping(document: &mut EditorDocument, index: usize, snap_enabled: bool) {
    if !snap_enabled {
        return;
    }

    let Some(entity) = document.level.entities.get(index).cloned() else {
        return;
    };

    let size = document
        .entity_types
        .get(&entity.entity_type)
        .map(|et| et.size())
        .unwrap_or(Vec2::ZERO);

    let a_left = entity.x;
    let a_right = entity.x + size.x;
    let a_bottom = entity.y;
    let a_top = entity.y + size.y;

    // Track best horizontal/vertical candidates. For horizontal we also remember
    // which other entity produced the candidate so we can check corner-snapping
    // along that same other entity after applying the edge snap.
    let mut best_dx: Option<(f32, usize)> = None; // (dx, other_index)
    let mut best_dy: Option<(f32, usize)> = None; // (dy, other_index)

    for (j, other) in document.level.entities.iter().enumerate() {
        if j == index {
            continue;
        }
        let Some(ot) = document.entity_types.get(&other.entity_type) else { continue; };
        let os = ot.size();
        let b_left = other.x;
        let b_right = other.x + os.x;
        let b_bottom = other.y;
        let b_top = other.y + os.y;

        // horizontal candidates: align left/right edges in multiple combinations
        let hx = [b_right - a_left, b_left - a_right, b_left - a_left, b_right - a_right];
        for &dx in &hx {
            if dx.abs() <= SNAP_THRESHOLD {
                if best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs() {
                    best_dx = Some((dx, j));
                }
            }
        }

        // vertical candidates: align bottom/top edges and parallels
        let hy = [b_top - a_bottom, b_bottom - a_top, b_bottom - a_bottom, b_top - a_top];
        for &dy in &hy {
            if dy.abs() <= SNAP_THRESHOLD {
                if best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs() {
                    best_dy = Some((dy, j));
                }
            }
        }
    }

    if best_dx.is_none() && best_dy.is_none() {
        return;
    }

    // Apply best horizontal snap first (if any). Remember which other entity
    // caused the snap so we can check for corner snapping along that edge.
    let mut corner_dy: Option<f32> = None;
    if let Some((dx, other_idx)) = best_dx {
        if dx.abs() > f32::EPSILON {
            if let Some(e) = document.level.entities.get_mut(index) {
                e.x += dx;
            }
        }

        // After an edge snap, check whether the corners are near the other's
        // corners along that edge and snap vertically if so.
        if let Some(other) = document.level.entities.get(other_idx) {
            if let Some(ot) = document.entity_types.get(&other.entity_type) {
                let os = ot.size();
                let _b_left = other.x;
                let _b_right = other.x + os.x;
                let b_bottom = other.y;
                let b_top = other.y + os.y;

                if let Some(e) = document.level.entities.get(index) {
                    let a_bottom = e.y;
                    let a_top = e.y + size.y;

                    // Candidate vertical adjustments between corners
                    let corner_candidates = [b_top - a_top, b_bottom - a_bottom, b_top - a_bottom, b_bottom - a_top];
                    for &dy in &corner_candidates {
                        if dy.abs() <= SNAP_THRESHOLD {
                            if corner_dy.is_none() || dy.abs() < corner_dy.unwrap().abs() {
                                corner_dy = Some(dy);
                            }
                        }
                    }
                }
            }
        }
    }

    // Decide vertical snap: prefer corner-based dy from the same entity that
    // produced the horizontal snap; otherwise fall back to the best independent dy.
    let dy_to_apply = corner_dy.or(best_dy.map(|t| t.0));
    if let Some(dy) = dy_to_apply {
        if dy.abs() > f32::EPSILON {
            if let Some(e) = document.level.entities.get_mut(index) {
                e.y += dy;
            }
        }
    }
}




pub(crate) fn topmost_entity_at_position(
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

pub(crate) fn camera_center_world(
    camera_query: &Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    let viewport_center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    camera.viewport_to_world_2d(camera_transform, viewport_center).ok()
}



