use bevy::prelude::*;
use bevy_egui::egui;

use super::hitbox::EntityTypeEditorState;

// Render the components sidebar. This was extracted from the parent module and
// uses helper functions defined in the parent via `super::` to avoid
// duplicating logic.
pub(crate) fn render_components_sidebar(
    ctx: &egui::Context,
    selected_name: &str,
    document: &mut Option<ResMut<crate::level::state::EditorDocument>>,
    entity_type_editor: &mut EntityTypeEditorState,
    fallback_entity_type: &crate::core::EntityTypeDefinition,
    mapping: &crate::level::state::ComponentValueMapping,
    mut toast: &mut crate::level::state::ToastState,
    time: &Time,
    mut widths: ResMut<crate::core::ColumnWidths>,
) {
    // Delegate to the original logic but qualify calls to parent helpers with
    // super:: so this module remains a thin extraction.

    egui::SidePanel::right("entity_type_components_sidebar")
        .resizable(true)
        .default_width(450.0)
        .min_width(300.0)
        .max_width(600.0)
        .show(ctx, |ui| {
            // Begin copy of original sidebar implementation; calls to helpers
            // are qualified with `super::` where necessary.
            ui.heading("Components");
            ui.add_space(6.0);

            let available_components = match crate::core::io::scan_game_components() {
                Ok(v) => v,
                Err(e) => {
                    toast.message = Some(format!("Could not scan components: {}", e));
                    toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    Vec::new()
                }
            };

            let staged_snapshot = super::helpers::cloned_staged_entity_type(
                document.as_deref(),
                entity_type_editor,
                selected_name,
                fallback_entity_type,
            );
            let components_snapshot = staged_snapshot.component_names();

            let add_options: Vec<String> = available_components
                .into_iter()
                .filter(|name| !components_snapshot.iter().any(|existing| existing == name))
                .collect();

            ui.horizontal(|ui| {
                ui.label("Add component:");
                let mut selected = entity_type_editor.add_selected.clone().unwrap_or_default();
                egui::ComboBox::from_id_salt(format!("add_component_cb_{}", selected_name))
                    .selected_text(if selected.is_empty() {
                        "select..."
                    } else {
                        &selected
                    })
                    .show_ui(ui, |ui| {
                        for option in &add_options {
                            ui.selectable_value(&mut selected, option.clone(), option);
                        }
                    });
                entity_type_editor.add_selected = if selected.is_empty() {
                    None
                } else {
                    Some(selected.clone())
                };

                let add_enabled = entity_type_editor
                    .add_selected
                    .as_ref()
                    .map(|selection| add_options.iter().any(|option| option == selection))
                    .unwrap_or(false);

                if ui
                    .add_enabled(
                        add_enabled,
                        egui::Button::new(egui_phosphor_icons::icons::PLUS),
                    )
                    .clicked()
                {
                    if let Some(chosen) = entity_type_editor.add_selected.clone() {
                        let mut new_components = components_snapshot.clone();
                        if !new_components.iter().any(|component| component == &chosen) {
                            new_components.push(chosen.clone());
                            if super::helpers::apply_to_staged_entity_type(
                                document.as_deref_mut(),
                                entity_type_editor,
                                selected_name,
                                fallback_entity_type,
                                |et| et.set_component_names(&new_components),
                            ) {
                                entity_type_editor
                                    .dirty_entity_types
                                    .insert(selected_name.to_string());
                            }
                        }
                    }
                    entity_type_editor.add_selected = None;
                }
            });

            crate::core::components_overrides::render_entity_type_overrides_table(
                ui,
                ctx,
                selected_name,
                document,
                entity_type_editor,
                fallback_entity_type,
                mapping,
                toast,
                time,
                &mut widths,
            );
        });
}
