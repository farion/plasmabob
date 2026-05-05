use bevy::asset::AssetServer;
use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::KeyCode;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::LevelFile;
use crate::entity_type;
use crate::level::helper::{
    apply_snapping, entity_render_center, is_inside_level_bounds, is_player_entity_type,
    topmost_entity_at_position, z_overlay_color_for_value,
};
use crate::level::run::{EditorCamera, RenderedLevelEntity, RenderedZOverlay};
use crate::level::state::{
    ClipboardEntity, EditorDocument, EditorUiState, EntityTypeViewState, EntityTypesSyncState,
    HitboxOverlayState, PointerState, SceneDirty, SelectionState, SnapState, ToastState,
    UndoCaptureState, UndoHistory, ZOverlayMode,
};

pub fn update_pointer_world_position(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut pointer_state: ResMut<PointerState>,
) {
    let Ok(window) = window_query.single() else {
        // if we cannot access the window, just clear the editor pointer position
        // stored in our local PointerState resource (not bevy::PointerState)
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

pub fn toggle_add_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<EditorUiState>,
    mut selection: ResMut<SelectionState>,
) {
    if keys.just_pressed(KeyCode::KeyA) {
        ui_state.show_add_menu = !ui_state.show_add_menu;
        selection.is_dragging = false;
    }
}

pub fn toggle_keyboard_legend_overlay(
    mut key_events: MessageReader<KeyboardInput>,
    mut ui_state: ResMut<EditorUiState>,
) {
    if logical_char_just_pressed(&mut key_events, "l") {
        ui_state.show_keyboard_legend_overlay = !ui_state.show_keyboard_legend_overlay;
    }
}

pub fn toggle_z_overlay_mode(
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

pub fn toggle_hitbox_overlay(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut hitbox_overlay: ResMut<HitboxOverlayState>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if control_pressed {
        return;
    }

    if !logical_char_just_pressed(&mut key_events, "h") {
        return;
    }

    hitbox_overlay.enabled = !hitbox_overlay.enabled;
}

pub fn toggle_snap(
    mut key_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut snap_state: ResMut<SnapState>,
    mut toast: ResMut<ToastState>,
    time: Res<Time>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if control_pressed {
        return;
    }

    if !logical_char_just_pressed(&mut key_events, "s") {
        return;
    }

    snap_state.enabled = !snap_state.enabled;
    toast.message = Some(if snap_state.enabled {
        "Snap: on".to_string()
    } else {
        "Snap: off".to_string()
    });
    toast.expires_at_seconds = time.elapsed_secs_f64() + 1.2;
}

pub fn logical_char_just_pressed(
    key_events: &mut MessageReader<KeyboardInput>,
    target: &str,
) -> bool {
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

pub fn undo_shortcut(
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

pub fn copy_entity_shortcut(
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
        .map(is_player_entity_type)
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

pub fn paste_entity_shortcut(
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

    // push undo
    if undo_history.states.len() >= 100 {
        undo_history.states.pop_front();
    }
    undo_history.states.push_back(document.level.clone());

    let mut new_entity = original_entity.clone();
    new_entity.id =
        crate::core::io::next_entity_id(&new_entity.entity_type, &document.level.entities);
    new_entity.x += 50.0;
    new_entity.y += 50.0;

    document.level.entities.push(new_entity);
    selection.selected_index = Some(document.level.entities.len() - 1);
    document.dirty = true;
    scene_dirty.0 = true;

    toast.message = Some("Entity inserted".to_string());
    toast.expires_at_seconds = time.elapsed_secs_f64() + 1.5;
}

pub fn save_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut toast: ResMut<ToastState>,
    mut document: ResMut<EditorDocument>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !control_pressed || !keys.just_pressed(KeyCode::KeyS) {
        return;
    }

    match crate::core::io::save_level(&document.level_fs_path, &document.level) {
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

pub fn delete_selected_entity_shortcut(
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

    if undo_history.states.len() >= 100 {
        undo_history.states.pop_front();
    }
    undo_history.states.push_back(document.level.clone());

    document.level.entities.remove(index);
    document.dirty = true;
    scene_dirty.0 = true;
    selection.selected_index = None;
    selection.is_dragging = false;
    selection.drag_offset = Vec2::ZERO;
}

pub fn adjust_selected_entity_z_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<EditorUiState>,
    selection: Res<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut rendered_entities: Query<(&RenderedLevelEntity, &mut Transform), Without<RenderedZOverlay>>,
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

    if undo_history.states.len() >= 100 {
        undo_history.states.pop_front();
    }
    undo_history.states.push_back(document.level.clone());
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

pub fn select_entity_on_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer_state: Res<PointerState>,
    ui_state: Res<EditorUiState>,
    document: Res<EditorDocument>,
    mut selection: ResMut<SelectionState>,
) {
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ui_state.show_add_menu
        || pointer_state.over_ui
        || !mouse_buttons.just_pressed(MouseButton::Left)
        || control_pressed
    {
        // when control is pressed we don't start a selection because Ctrl+Left
        // should be used for camera panning
        return;
    }

    let Some(pointer_world) = pointer_state.world_position else {
        return;
    };

    // Use the level helper directly.
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

pub fn drag_selected_entity(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer_state: Res<PointerState>,
    mut selection: ResMut<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut capture_state: ResMut<UndoCaptureState>,
    mut rendered_entities: Query<(&RenderedLevelEntity, &mut Transform), Without<RenderedZOverlay>>,
    mut rendered_overlays: Query<(&RenderedZOverlay, &mut Transform), Without<RenderedLevelEntity>>,
    snap_state: Res<SnapState>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) {
        selection.is_dragging = false;
        capture_state.drag_snapshot_taken = false;
        return;
    }

    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if control_pressed {
        // If control is held we use left-drag to pan the camera, so do not
        // drag the selected entity.
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
    if (new_position - old_position).length_squared() > f32::EPSILON
        && !capture_state.drag_snapshot_taken
    {
        if undo_history.states.len() >= 100 {
            undo_history.states.pop_front();
        }
        undo_history.states.push_back(document.level.clone());
        capture_state.drag_snapshot_taken = true;
    }

    if let Some(entity) = document.level.entities.get_mut(index) {
        entity.x = new_position.x;
        entity.y = new_position.y;
    } else {
        return;
    }
    // Apply snapping after updating the entity position.
    apply_snapping(&mut document, index, snap_state.enabled);
    document.dirty = true;

    let (render_position, _) = if let Some(e) = document.level.entities.get(index) {
        let size = document
            .entity_types
            .get(&e.entity_type)
            .map(|entity_type| entity_type.size())
            .unwrap_or(Vec2::ZERO);
        (entity_render_center(Vec2::new(e.x, e.y), size), size)
    } else {
        (entity_render_center(new_position, Vec2::ZERO), Vec2::ZERO)
    };

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

pub fn move_selected_entity_with_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<EditorUiState>,
    selection: Res<SelectionState>,
    mut document: ResMut<EditorDocument>,
    mut undo_history: ResMut<UndoHistory>,
    mut capture_state: ResMut<UndoCaptureState>,
    mut rendered_entities: Query<(&RenderedLevelEntity, &mut Transform), Without<RenderedZOverlay>>,
    mut rendered_overlays: Query<(&RenderedZOverlay, &mut Transform), Without<RenderedLevelEntity>>,
    snap_state: Res<SnapState>,
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
        if undo_history.states.len() >= 100 {
            undo_history.states.pop_front();
        }
        undo_history.states.push_back(document.level.clone());
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
    apply_snapping(&mut document, index, snap_state.enabled);
    document.dirty = true;

    let (render_position, _) = if let Some(e) = document.level.entities.get(index) {
        let size = document
            .entity_types
            .get(&e.entity_type)
            .map(|entity_type| entity_type.size())
            .unwrap_or(Vec2::ZERO);
        (entity_render_center(Vec2::new(e.x, e.y), size), size)
    } else {
        (
            entity_render_center(Vec2::new(new_x, new_y), Vec2::ZERO),
            Vec2::ZERO,
        )
    };

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

pub fn camera_controls(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
    pointer_state: Res<crate::level::state::PointerState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    // If the pointer is currently over UI we should not let camera controls
    // (zoom or pan) react to mouse input — prevents scrolling the sidebar
    // from zooming the world underneath.
    if pointer_state.over_ui {
        mouse_motion.clear();
        mouse_wheel.clear();
        return;
    }

    let current_scale = match projection.as_mut() {
        Projection::Orthographic(orthographic) => orthographic.scale,
        _ => 1.0,
    };

    // Right mouse pan OR Ctrl + Left mouse pan
    let control_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if mouse_buttons.pressed(MouseButton::Right)
        || (control_pressed && mouse_buttons.pressed(MouseButton::Left))
    {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, event| acc + event.delta);
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
