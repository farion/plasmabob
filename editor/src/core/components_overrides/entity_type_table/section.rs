use bevy::prelude::*;
use bevy_egui::egui;

use super::layout::TableLayout;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_component_section(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    selected_name: &str,
    component_name: &str,
    staged_snapshot: &crate::core::EntityTypeDefinition,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    mapping: &crate::level::state::ComponentValueMapping,
    toast: &mut crate::level::state::ToastState,
    time: &Time,
    widths: &mut crate::core::ColumnWidths,
    layout: &TableLayout,
) {
    let attr_rows = crate::entity_type::sorted_attribute_rows(
        mapping,
        staged_snapshot,
        fallback_entity_type,
        component_name,
    );

    let component_scope_id = format!(
        "entity_type_component_section_{}_{}",
        selected_name, component_name
    );
    ui.push_id(component_scope_id, |ui| {
        ui.add_space(4.0);
        let header_h = 24.0f32;
        let header_size = egui::vec2(ui.available_width(), header_h);
        let (header_rect, _header_resp) = ui.allocate_exact_size(header_size, egui::Sense::click());

        let is_collapsed = entity_type_editor
            .collapsed_components
            .contains(component_name);
        let left_rect = egui::Rect::from_min_max(
            header_rect.min,
            egui::pos2(header_rect.max.x - layout.clear_col_w, header_rect.max.y),
        );
        let arrow_icon = if is_collapsed {
            egui_phosphor_icons::icons::CARET_RIGHT
        } else {
            egui_phosphor_icons::icons::CARET_DOWN
        };
        let arrow_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.min.x + 4.0, left_rect.min.y),
            egui::pos2(left_rect.min.x + 4.0 + layout.button_w, left_rect.max.y),
        );
        let arrow_resp = ui.put(
            arrow_rect,
            egui::Button::new(arrow_icon).min_size(egui::vec2(layout.button_w, header_h)),
        );
        if arrow_resp.clicked() {
            if is_collapsed {
                entity_type_editor
                    .collapsed_components
                    .remove(component_name);
            } else {
                entity_type_editor
                    .collapsed_components
                    .insert(component_name.to_string());
            }
        }

        let header_label_offset = arrow_rect.max.x - header_rect.min.x + 6.0;
        let label_rect = egui::Rect::from_min_max(
            egui::pos2(header_rect.min.x + header_label_offset, left_rect.min.y),
            egui::pos2(left_rect.max.x, left_rect.max.y),
        );
        ui.allocate_ui_at_rect(label_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(component_name).strong(),
                ));
            });
        });

        let right_rect = egui::Rect::from_min_max(
            egui::pos2(header_rect.max.x - layout.clear_col_w, header_rect.min.y),
            header_rect.max,
        );
        let btn_center_x = (right_rect.min.x + right_rect.max.x) * 0.5;
        let button_rect = egui::Rect::from_min_max(
            egui::pos2(btn_center_x - layout.button_w * 0.5, right_rect.min.y),
            egui::pos2(btn_center_x + layout.button_w * 0.5, right_rect.max.y),
        );
        let trash_resp = ui.put(
            button_rect,
            egui::Button::new(egui_phosphor_icons::icons::TRASH)
                .min_size(egui::vec2(layout.button_w, header_h)),
        );

        ui.add_space(6.0);
        if trash_resp.clicked() {
            entity_type_editor.remove_component_confirm = Some(component_name.to_string());
        }

        if !entity_type_editor
            .collapsed_components
            .contains(component_name)
        {
            widths.widths = vec![layout.name_col_w, layout.middle_col_w, layout.clear_col_w];
            super::rows::render_component_attributes_table(
                ui,
                selected_name,
                component_name,
                &attr_rows,
                staged_snapshot,
                document,
                entity_type_editor,
                fallback_entity_type,
                layout,
                header_label_offset,
            );
        }

        ui.separator();
        super::cells::handle_component_array_modal(
            ctx,
            component_name,
            selected_name,
            document,
            entity_type_editor,
            fallback_entity_type,
            toast,
            time,
        );
    });
}
