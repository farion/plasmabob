use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use egui_phosphor_icons::add_fonts;
use serde::{Deserialize, Serialize};

use crate::core::io::assets_dir;
use crate::level::helper::*;
use crate::level::state::*;
// Explicit single import to disambiguate PointerState from bevy's PointerState
use crate::level::state::PointerState;

// Editor entry and a few small helpers moved from crate::editor into the
// level module so level-specific code lives together. Keep definitions
// at crate::level and re-export at crate::editor for backwards compatibility.

#[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActiveCharacter {
    Bob,
    Betty,
}

impl Default for ActiveCharacter {
    fn default() -> Self {
        ActiveCharacter::Bob
    }
}

impl ActiveCharacter {
    pub fn load_from_disk() -> Self {
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

    pub fn save_to_disk(&self) -> Result<(), io::Error> {
        let path = Self::settings_path();
        let obj = serde_json::json!({"active_character": match self { ActiveCharacter::Bob => "bob", ActiveCharacter::Betty => "betty" }});
        std::fs::write(
            path,
            serde_json::to_string_pretty(&obj).unwrap_or_else(|_| String::new()),
        )
    }

    fn settings_path() -> PathBuf {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return parent.join("editor_settings.json");
            }
        }
        crate::core::io::workspace_root().join("editor_settings.json")
    }

    pub fn toggle(&mut self) {
        *self = match self {
            ActiveCharacter::Bob => ActiveCharacter::Betty,
            ActiveCharacter::Betty => ActiveCharacter::Bob,
        }
    }
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum EditorMode {
    #[default]
    LevelPicker,
    Editing,
    EntityTypeView,
}

#[derive(Component)]
pub struct EditorCamera;

#[derive(Component)]
pub struct SceneEntity;

#[derive(Component)]
pub struct RenderedLevelEntity {
    pub index: usize,
}

#[derive(Component)]
pub struct RenderedZOverlay {
    pub index: usize,
}

#[derive(Component)]
pub struct PendingBackgroundTiles {
    pub image: Handle<Image>,
    pub level_size: Vec2,
}

#[derive(Component)]
pub struct BackgroundTilesReady;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, EditorCamera));
}

fn setup_phosphor_fonts(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        let mut fonts = egui::FontDefinitions::default();
        add_fonts(&mut fonts);
        ctx.set_fonts(fonts);
    }
}

fn refresh_level_catalog(mut catalog: ResMut<LevelCatalog>) {
    match crate::core::io::scan_worlds() {
        Ok(worlds) => {
            catalog.worlds = worlds;
        }
        Err(_) => {
            catalog.worlds.clear();
        }
    }

    match crate::core::io::scan_levels() {
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

// Checks the background entity-type sync thread result and surfaces a brief
// toast / error in the UI if work completed. This was previously a small
// helper inside editor.rs; keep it local to the level module for now.
fn check_sync_result(
    mut sync_state: ResMut<EntityTypesSyncState>,
    mut catalog: ResMut<LevelCatalog>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    if let Ok(mut guard) = sync_state.result.lock() {
        if let Some(res) = guard.take() {
            match res {
                Ok(report) => {
                    catalog.error = None;
                    toast.message = Some(format!(
                        "Entity types regenerated: {} files",
                        report.updated + report.deleted
                    ));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                }
                Err(err) => {
                    catalog.error = Some(err.clone());
                    toast.message = Some(format!("Entity types sync failed: {}", err));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 5.0;
                }
            }
        }
    }
}

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

    let dot_size = 1.0_f32;
    let spacing = 4.0_f32;
    let speed = 20.0_f64;

    let left = entity.x;
    let bottom = entity.y;
    let right = entity.x + size.x;
    let top = entity.y + size.y;

    let perim = 2.0 * (size.x + size.y);
    let t = time.elapsed_secs_f64();
    let offset = ((t * speed) % (spacing as f64)) as f32;

    let point_at = |mut dist: f32| -> Vec2 {
        dist = dist % perim;
        if dist <= size.x {
            Vec2::new(left + dist, bottom)
        } else if dist <= size.x + size.y {
            Vec2::new(right, bottom + (dist - size.x))
        } else if dist <= size.x + size.y + size.x {
            Vec2::new(right - (dist - (size.x + size.y)), top)
        } else {
            Vec2::new(left, top - (dist - (size.x + size.y + size.x)))
        }
    };

    let mut dist = offset;
    while dist < perim {
        let p = point_at(dist);
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
                        max_x = box_pts
                            .iter()
                            .map(|p| p[0])
                            .fold(f32::NEG_INFINITY, f32::max);
                        min_y = box_pts.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                        max_y = box_pts
                            .iter()
                            .map(|p| p[1])
                            .fold(f32::NEG_INFINITY, f32::max);
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
        gizmos.rect_2d(
            hit_center,
            Vec2::new(hit_w, hit_h),
            Color::srgb(1.0, 0.0, 0.0),
        );
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
        gizmos.rect_2d(
            center,
            size + Vec2::splat(6.0),
            Color::srgba(1.0, 0.3, 0.3, 0.4),
        );
    }
}

pub fn run() {
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
        .init_resource::<crate::core::ColumnWidths>()
        .insert_resource(ActiveCharacter::load_from_disk())
        .init_state::<EditorMode>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, crate::level::state::setup_component_value_mapping)
        .add_systems(EguiPrimaryContextPass, setup_phosphor_fonts)
        .add_systems(OnEnter(EditorMode::LevelPicker), refresh_level_catalog)
        .add_systems(
            EguiPrimaryContextPass,
            crate::level::level_picker_ui.run_if(in_state(EditorMode::LevelPicker)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            crate::entity_type::entity_type_view_ui.run_if(in_state(EditorMode::EntityTypeView)),
        )
        .add_systems(
            Update,
            check_sync_result.run_if(in_state(EditorMode::LevelPicker)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            crate::level::editing_ui.run_if(in_state(EditorMode::Editing)),
        )
        .add_systems(
            Update,
            crate::level::toggle_hitbox_overlay.run_if(in_state(EditorMode::Editing)),
        )
        .add_systems(
            Update,
            draw_hitbox_outlines.run_if(in_state(EditorMode::Editing)),
        )
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
