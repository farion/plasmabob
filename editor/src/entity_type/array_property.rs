use super::array_editor::{
    format_array_short, inner_array_value_to_csv_string, parse_array_type_signature,
    ArrayEditorState,
};
use crate::entity_type::hitbox::EntityTypeEditorState;
use bevy_egui::egui;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ARRAY_EDITOR_SESSION_ID: AtomicU64 = AtomicU64::new(1);

// Renders the compact sidebar view for an attribute whose type starts with "array".
// Shows a short representation and a pencil button that initializes the array
// editor state stored in entity_type_editor.array_editor.
pub(crate) fn render_array_property(
    ui: &mut egui::Ui,
    row_name: &str,
    attr_type: &str,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    selected_name: &str,
    document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let mut values = explicit_value
        .and_then(|v| v.as_array().cloned())
        .or_else(|| display_default.and_then(|v| v.as_array().cloned()))
        .unwrap_or_default();

    // compute a short JSON-like repr using helper from array_editor
    let short = format_array_short(&values);
    // Allocate the full middle-cell area to control exact placement of the
    // edit button and the short text immediately after it.
    let cell_h = 20.0f32;
    let cell_size = egui::vec2(middle_col_w, cell_h);
    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());

    // Button at exact left of the cell
    let btn_w = 26.0f32;
    let btn_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.min.x, cell_rect.min.y),
        egui::pos2(cell_rect.min.x + btn_w, cell_rect.max.y),
    );
    let btn_resp = ui.put(
        btn_rect,
        egui::Button::new(egui_phosphor_icons::icons::PENCIL_SIMPLE)
            .min_size(egui::vec2(btn_w, cell_h)),
    );
    if btn_resp.clicked() {
        // initialize array editor state
        let type_desc = format!("{}.{} {}", component_name, row_name, attr_type);
        let parsed = parse_array_type_signature(attr_type);
        let mut inner_edit_strings = Vec::new();
        for v in &values {
            if parsed.element_is_array {
                inner_edit_strings.push(inner_array_value_to_csv_string(v));
            } else {
                inner_edit_strings.push(match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    other => serde_json::to_string(other).unwrap_or_default(),
                });
            }
        }

        entity_type_editor.array_editor = Some(ArrayEditorState {
            component_name: component_name.to_string(),
            attr_name: row_name.to_string(),
            display_type: type_desc,
            values: values.clone(),
            original: values.clone(),
            element_is_array: parsed.element_is_array,
            element_is_number: parsed.element_is_number,
            inner_fixed_len: parsed.inner_fixed_len,
            outer_fixed_len: parsed.outer_fixed_len,
            inner_edit_strings,
            modal_pos: egui::pos2(0.0, 0.0),
            modal_size: egui::vec2(500.0, 300.0),
            modal_initialized: false,
            window_session_id: NEXT_ARRAY_EDITOR_SESSION_ID.fetch_add(1, Ordering::Relaxed),
        });
    }

    // Text area immediately after button with a small margin
    let margin = 6.0f32;
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(btn_rect.max.x + margin, cell_rect.min.y),
        cell_rect.max,
    );
    ui.allocate_ui_at_rect(text_rect, |ui| {
        let text_w = text_rect.width().max(20.0);
        let avg_char_w = 7.5_f32 * ui.ctx().pixels_per_point();
        let max_chars = (text_w / avg_char_w) as usize;
        let mut displayed = short.clone();
        if displayed.chars().count() > max_chars && max_chars > 1 {
            displayed = displayed
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
                + "…";
        }
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.add(egui::Label::new(egui::RichText::new(displayed)));
        });
    });
}
