use bevy_egui::egui;
use serde_json::Value;
// session id generation moved to core::next_window_session_id()

use super::{
    format_array_short, inner_array_value_to_csv_string, parse_array_type_signature,
    ArrayEditorState,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_array_property(
    ui: &mut egui::Ui,
    row_name: &str,
    attr_type: &str,
    explicit_value: Option<&Value>,
    display_default: Option<&Value>,
    middle_col_w: f32,
    component_name: &str,
    _selected_name: &str,
    _document: Option<&mut crate::level::state::EditorDocument>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    _fallback_entity_type: &crate::core::EntityTypeDefinition,
) {
    let values = explicit_value
        .and_then(|v| v.as_array().cloned())
        .or_else(|| display_default.and_then(|v| v.as_array().cloned()))
        .unwrap_or_default();

    // delegate layout and rendering to helper below

    // Reuse shared helper for rendering the edit button + short text so
    // entity-type and level overrides use identical behaviour.
    let short = format_array_short(&values);
    let cell_h = 20.0f32;
    let cell_size = egui::vec2(middle_col_w, cell_h);
    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());

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
            window_session_id: crate::core::next_window_session_id(),
        });
    }

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

/// Helper shared within this module; exported by components_overrides to be
/// reused by level_table so both UIs show a consistent edit affordance.
pub(crate) fn render_array_edit_button_and_short(
    ui: &mut egui::Ui,
    component_name: &str,
    attr_name: &str,
    attr_type: &str,
    values: &[Value],
    text_area_w: f32,
    array_editor: &mut Option<ArrayEditorState>,
) {
    let btn_w = 26.0f32;
    let cell_h = 20.0f32;
    let cell_size = egui::vec2(text_area_w, cell_h);
    let (cell_rect, _cell_resp) = ui.allocate_exact_size(cell_size, egui::Sense::hover());

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
        let type_desc = format!("{}.{} {}", component_name, attr_name, attr_type);
        let parsed = parse_array_type_signature(attr_type);
        let mut inner_edit_strings = Vec::new();
        for v in values {
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

        *array_editor = Some(ArrayEditorState {
            component_name: component_name.to_string(),
            attr_name: attr_name.to_string(),
            display_type: type_desc,
            values: values.to_vec(),
            original: values.to_vec(),
            element_is_array: parsed.element_is_array,
            element_is_number: parsed.element_is_number,
            inner_fixed_len: parsed.inner_fixed_len,
            outer_fixed_len: parsed.outer_fixed_len,
            inner_edit_strings,
            modal_pos: egui::pos2(0.0, 0.0),
            modal_size: egui::vec2(500.0, 300.0),
            modal_initialized: false,
            window_session_id: crate::core::next_window_session_id(),
        });
    }

    let margin = 6.0f32;
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(btn_rect.max.x + margin, cell_rect.min.y),
        cell_rect.max,
    );
    ui.allocate_ui_at_rect(text_rect, |ui| {
        let text_w = text_rect.width().max(20.0);
        let avg_char_w = 7.5_f32 * ui.ctx().pixels_per_point();
        let max_chars = (text_w / avg_char_w) as usize;
        let mut displayed = format_array_short(values);
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
