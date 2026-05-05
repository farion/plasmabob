use bevy_egui::egui;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::core::components_overrides::ArrayEditorState;

#[derive(Clone, Copy, Debug)]
pub(crate) enum DragEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveHitboxDrag {
    pub state_key: String,
    pub edge: DragEdge,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RectHitbox {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl RectHitbox {
    pub fn from_points(
        points: &[[f32; 2]],
        image_w: f32,
        image_h: f32,
        units_per_pixel: f32,
    ) -> Self {
        let max_w_units = image_w.max(super::HITBOX_MIN_SIZE_PX) * units_per_pixel;
        let max_h_units = image_h.max(super::HITBOX_MIN_SIZE_PX) * units_per_pixel;

        if points.is_empty() {
            return Self {
                left: 0.0,
                right: max_w_units,
                bottom: 0.0,
                top: max_h_units,
            };
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for [x, y] in points {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }

        let mut rect = Self {
            left: min_x,
            right: max_x,
            bottom: min_y,
            top: max_y,
        };
        rect.clamp_to_image(image_w, image_h, units_per_pixel);
        rect
    }

    pub fn clamp_to_image(&mut self, image_w: f32, image_h: f32, units_per_pixel: f32) {
        let min_size_units = super::HITBOX_MIN_SIZE_PX * units_per_pixel;
        let max_w = image_w.max(super::HITBOX_MIN_SIZE_PX) * units_per_pixel;
        let max_h = image_h.max(super::HITBOX_MIN_SIZE_PX) * units_per_pixel;

        self.left = self.left.clamp(0.0, max_w - min_size_units);
        self.right = self.right.clamp(self.left + min_size_units, max_w);
        self.bottom = self.bottom.clamp(0.0, max_h - min_size_units);
        self.top = self.top.clamp(self.bottom + min_size_units, max_h);
    }

    pub fn drag_edge(
        &mut self,
        edge: DragEdge,
        units_delta: egui::Vec2,
        image_w: f32,
        image_h: f32,
        units_per_pixel: f32,
    ) {
        match edge {
            DragEdge::Left => {
                self.left += units_delta.x;
            }
            DragEdge::Right => {
                self.right += units_delta.x;
            }
            DragEdge::Bottom => {
                self.bottom += units_delta.y;
            }
            DragEdge::Top => {
                self.top += units_delta.y;
            }
        }
        self.clamp_to_image(image_w, image_h, units_per_pixel);
    }

    pub fn to_json_points(self) -> [[f32; 2]; 4] {
        [
            [self.left, self.bottom],
            [self.right, self.bottom],
            [self.right, self.top],
            [self.left, self.top],
        ]
    }
}

pub(crate) struct EntityTypeEditorState {
    pub show_hitboxes: bool,
    pub active_drag: Option<ActiveHitboxDrag>,
    pub edited_hitboxes: HashMap<String, RectHitbox>,
    pub dirty_states: HashSet<String>,
    pub last_entity_type: Option<String>,
    pub add_selected: Option<String>,
    pub remove_component_confirm: Option<String>,
    pub collapsed_components: HashSet<String>,
    pub json_editor_state: HashMap<String, String>,
    pub dirty_entity_types: HashSet<String>,
    pub edited_entity_types: HashMap<String, crate::core::EntityTypeDefinition>,
    // Optional state when editing an array property in a modal.
    pub array_editor: Option<ArrayEditorState>,
}

impl Default for EntityTypeEditorState {
    fn default() -> Self {
        Self {
            // Enable hitbox overlays by default per user request
            show_hitboxes: true,
            active_drag: None,
            edited_hitboxes: HashMap::new(),
            dirty_states: HashSet::new(),
            last_entity_type: None,
            add_selected: None,
            remove_component_confirm: None,
            collapsed_components: HashSet::new(),
            json_editor_state: HashMap::new(),
            dirty_entity_types: HashSet::new(),
            edited_entity_types: HashMap::new(),
            array_editor: None,
        }
    }
}

pub(crate) fn hitbox_to_screen(
    rect: RectHitbox,
    image_rect: egui::Rect,
    image_size: egui::Vec2,
) -> egui::Rect {
    let x_scale = if image_size.x > 0.0 {
        image_rect.width() / image_size.x
    } else {
        1.0
    };
    let y_scale = if image_size.y > 0.0 {
        image_rect.height() / image_size.y
    } else {
        1.0
    };

    let left = image_rect.left() + rect.left * x_scale;
    let right = image_rect.left() + rect.right * x_scale;
    let bottom = image_rect.bottom() - rect.bottom * y_scale;
    let top = image_rect.bottom() - rect.top * y_scale;

    egui::Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom))
}

pub(crate) fn units_per_pixel(configured_height: f32, image_height: f32) -> f32 {
    if configured_height > 0.0 && image_height > 0.0 {
        configured_height / image_height
    } else {
        1.0
    }
}

pub(crate) fn hitbox_to_screen_with_ratio(
    rect_units: RectHitbox,
    image_rect: egui::Rect,
    image_size_pixels: egui::Vec2,
    units_per_pixel_value: f32,
) -> egui::Rect {
    let inv = if units_per_pixel_value > 0.0 {
        1.0 / units_per_pixel_value
    } else {
        1.0
    };
    let rect_pixels = RectHitbox {
        left: rect_units.left * inv,
        right: rect_units.right * inv,
        bottom: rect_units.bottom * inv,
        top: rect_units.top * inv,
    };

    hitbox_to_screen(rect_pixels, image_rect, image_size_pixels)
}

pub(crate) fn pick_hitbox_edge(pointer: egui::Pos2, screen_hitbox: egui::Rect) -> Option<DragEdge> {
    if !screen_hitbox
        .expand(super::HITBOX_EDGE_PICK_TOLERANCE_PX)
        .contains(pointer)
    {
        return None;
    }

    let mut candidates = [
        (DragEdge::Left, (pointer.x - screen_hitbox.left()).abs()),
        (DragEdge::Right, (pointer.x - screen_hitbox.right()).abs()),
        (DragEdge::Bottom, (pointer.y - screen_hitbox.bottom()).abs()),
        (DragEdge::Top, (pointer.y - screen_hitbox.top()).abs()),
    ];
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

    if candidates[0].1 <= super::HITBOX_EDGE_PICK_TOLERANCE_PX {
        Some(candidates[0].0)
    } else {
        None
    }
}

pub(crate) fn cursor_for_drag_edge(edge: DragEdge) -> egui::CursorIcon {
    match edge {
        DragEdge::Left | DragEdge::Right => egui::CursorIcon::ResizeHorizontal,
        DragEdge::Top | DragEdge::Bottom => egui::CursorIcon::ResizeVertical,
    }
}
