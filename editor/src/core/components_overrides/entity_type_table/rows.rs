use bevy::prelude::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use serde_json::Value;

use super::layout::TableLayout;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_component_attributes_table(
    ui: &mut egui::Ui,
    selected_name: &str,
    component_name: &str,
    attr_rows: &[crate::entity_type::AttributeUiRow],
    staged_snapshot: &crate::core::EntityTypeDefinition,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    layout: &TableLayout,
    header_label_offset: f32,
) {
    let table = TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(layout.name_col_w))
        .column(Column::exact(layout.middle_col_w))
        .column(Column::exact(layout.clear_col_w));

    table.body(|mut body| {
        for row in attr_rows {
            let explicit_value =
                staged_snapshot.component_attribute_value(component_name, &row.name);
            let component_default =
                crate::entity_type::component_default_value(component_name, &row.name);
            let enum_default = if row.attr_type == "enum" {
                row.options.first().cloned().map(Value::String)
            } else {
                None
            };
            let display_default = component_default.clone().or(enum_default.clone());

            body.row(20.0, |mut r| {
                r.col(|ui| {
                    super::cells::render_attribute_name_cell(
                        ui,
                        &row.name,
                        layout.name_col_w,
                        header_label_offset,
                    );
                });
                r.col(|ui| {
                    super::cells::render_attribute_value_cell(
                        ui,
                        row,
                        &explicit_value,
                        display_default.as_ref(),
                        layout.middle_col_w,
                        component_name,
                        selected_name,
                        document,
                        entity_type_editor,
                        fallback_entity_type,
                    );
                });
                r.col(|ui| {
                    super::cells::render_attribute_reset_cell(
                        ui,
                        explicit_value.is_some(),
                        layout,
                        component_name,
                        &row.name,
                        selected_name,
                        document,
                        entity_type_editor,
                        fallback_entity_type,
                    );
                });
            });

            if explicit_value.is_none() {
                render_default_hint_row(
                    &mut body,
                    component_default.is_some(),
                    enum_default.is_some(),
                );
            }
        }
    });
}

fn render_default_hint_row(
    body: &mut egui_extras::TableBody<'_>,
    has_component_default: bool,
    has_enum_default: bool,
) {
    let hint = if has_component_default {
        Some("component default")
    } else if has_enum_default {
        Some("first enum option")
    } else {
        None
    };
    if let Some(source) = hint {
        body.row(20.0, |mut rr| {
            rr.col(|ui| {
                ui.label("");
            });
            rr.col(|ui| {
                ui.label(egui::RichText::new("default").weak().italics())
                    .on_hover_text(source);
            });
            rr.col(|ui| {
                ui.label("");
            });
        });
    }
}
