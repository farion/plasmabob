use bevy::prelude::*;
use bevy_egui::egui;

use super::layout::compute_layout;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_entity_type_overrides_table(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut crate::entity_type::EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    mapping: &crate::level::state::ComponentValueMapping,
    toast: &mut crate::level::state::ToastState,
    time: &Time,
    widths: &mut crate::core::ColumnWidths,
) {
    let staged_snapshot = crate::entity_type::cloned_staged_entity_type(
        document.as_deref(),
        entity_type_editor,
        selected_name,
        fallback_entity_type,
    );
    let components_snapshot = staged_snapshot.component_names();

    let max_name_chars = max_name_chars_for_components(
        &components_snapshot,
        mapping,
        &staged_snapshot,
        fallback_entity_type,
    );
    let layout = compute_layout(ui, widths, max_name_chars);
    widths.widths = vec![layout.name_col_w, layout.middle_col_w, layout.clear_col_w];

    egui::ScrollArea::vertical()
        .id_salt(format!("entity_type_components_scroll_{}", selected_name))
        .show(ui, |ui| {
            for component_name in &components_snapshot {
                super::section::render_component_section(
                    ui,
                    ctx,
                    selected_name,
                    component_name,
                    &staged_snapshot,
                    document,
                    entity_type_editor,
                    fallback_entity_type,
                    mapping,
                    toast,
                    time,
                    widths,
                    &layout,
                );
            }
        });
}

fn max_name_chars_for_components(
    components_snapshot: &[String],
    mapping: &crate::level::state::ComponentValueMapping,
    staged_snapshot: &crate::core::EntityTypeDefinition,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
) -> usize {
    let mut max_name_chars = 0usize;
    for comp in components_snapshot {
        let rows = crate::entity_type::sorted_attribute_rows(
            mapping,
            staged_snapshot,
            fallback_entity_type,
            comp,
        );
        for r in rows {
            max_name_chars = max_name_chars.max(r.name.len());
        }
    }
    max_name_chars
}
