use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::model::EntityTypeDefinition;

pub(crate) fn entity_type_view_ui(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    view_state: Res<crate::editor::EntityTypeViewState>,
    document: Option<Res<crate::editor::EditorDocument>>,
    mut next_state: ResMut<NextState<crate::editor::EditorMode>>,
) {
    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Zurück").clicked() {
                next_state.set(crate::editor::EditorMode::LevelPicker);
            }
        });

        ui.separator();

        let selected = match &view_state.selected {
            Some(name) => name.clone(),
            None => {
                ui.label("Kein EntityType ausgewählt.");
                return;
            }
        };

        // The editor document may contain parsed entity types if a level is loaded.
        // Otherwise we try to load the entity type JSON from assets/entity_types/<name>.json
        let maybe_def: Option<EntityTypeDefinition> = if let Some(doc) = &document {
            doc.entity_types.get(&selected).cloned()
        } else {
            // Try load from file path
            let path = format!("entity_types/{}.json", selected);
            match std::fs::read_to_string(bevy::asset::AssetServer::get_handle_path(&asset_server.load::<Image, _>("dummy")).unwrap_or_default()) {
                _ => None,
            }
        };

        let def = match maybe_def {
            Some(d) => d,
            None => {
                ui.label(format!("EntityType '{}' nicht gefunden.", selected));
                return;
            }
        };

        ui.heading(format!("EntityType: {}", selected));
        ui.add_space(8.0);

        // For each state show a horizontal row of images
        for (state_name, state_def) in def.states.iter() {
            ui.group(|ui| {
                ui.label(state_name);
                ui.horizontal(|ui| {
                    for frame in &state_def.animation {
                        // Normalize path (strip assets/)
                        let asset_path = crate::model::normalize_asset_reference(frame);
                        let handle: Handle<Image> = asset_server.load(asset_path.clone());

                        // Try to get texture id for egui if available
                        let mut texture_id = None;
                        for (handle_ref, _image) in images.iter() {
                            // We don't have an easy mapping here; fall back to labeled button
                            let _ = handle_ref;
                        }

                        if let Some(id) = texture_id {
                            ui.image(id, [64.0, 64.0]);
                        } else {
                            if ui.button(format!("{}", frame)).clicked() {
                                // placeholder: maybe open image externally
                            }
                        }
                    }
                });
            });
            ui.add_space(6.0);
        }
    });
}

