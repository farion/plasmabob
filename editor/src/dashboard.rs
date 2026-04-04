// Minimal dashboard entry module used by `editor/src/main.rs`.
// It simply delegates to the editor run entrypoint so the editor starts in
// the same way. The actual render function for the level picker lives in
// `editor::dashboard` (module `src/editor/dashboard.rs`) where it can access
// editor-local types.

use bevy_egui::egui;

pub(crate) fn render_level_picker_columns(
    ui: &mut egui::Ui,
    open_asset_path: &mut Option<String>,
    catalog: &mut crate::editor::LevelCatalog,
    sync_state: &mut crate::editor::EntityTypesSyncState,
    entity_type_files: &Vec<String>,
    entity_type_error: &Option<String>,
) -> Option<String> {
    let mut selected: Option<String> = None;

    ui.columns(3, |columns| {
        columns[0].vertical(|ui| {
            ui.heading("Worlds");
            ui.add_space(8.0);
            if ui.button("Force Reread").clicked() {
                if let Ok(worlds) = crate::io::scan_worlds() {
                    catalog.worlds = worlds;
                }
                if let Ok(levels) = crate::io::scan_levels() {
                    catalog.levels = levels;
                    catalog.error = None;
                }
            }
            ui.add_space(8.0);

            let list_height = ui.available_height();
            ui.push_id("worlds_scroll_area", |ui| {
                egui::ScrollArea::vertical().max_height(list_height).show(ui, |ui| {
                    if catalog.worlds.is_empty() {
                        ui.label("No worlds present yet.");
                    } else {
                        for world in &catalog.worlds {
                            ui.push_id(format!("world_item:{}", world.asset_path), |ui| {
                                if ui.button(&world.display_name).clicked() {
                                    // set selected world to the folder name (stem of json file)
                                    let folder = std::path::Path::new(&world.asset_path)
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| s.to_string());
                                    catalog.selected_world = folder;
                                }
                            });
                        }
                    }
                });
            });
        });

        columns[1].vertical(|ui| {
            ui.heading("Levels");
            ui.add_space(8.0);
            if ui.button("Force Reread").clicked() {
                match crate::io::scan_levels() {
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
            ui.add_space(8.0);

            let list_height = ui.available_height();
            ui.push_id("levels_scroll_area", |ui| {
                egui::ScrollArea::vertical().max_height(list_height).show(ui, |ui| {
                    if let Some(error) = &catalog.error {
                        ui.colored_label(egui::Color32::RED, error);
                        return;
                    }

                    if catalog.worlds.is_empty() {
                        ui.label("No worlds found. Please press Force Reread.");
                        return;
                    }

                    let selected_world = catalog.selected_world.as_deref();
                    if selected_world.is_none() {
                        ui.label("No world selected.");
                        return;
                    }

                    let folder = selected_world.unwrap();
                    let prefix = format!("worlds/{}/", folder);
                    let filtered: Vec<&crate::io::LevelEntry> = catalog
                        .levels
                        .iter()
                        .filter(|lvl| lvl.asset_path.starts_with(&prefix))
                        .collect();

                    if filtered.is_empty() {
                        ui.label("No valid level files found for this world.");
                    } else {
                        for level in filtered {
                            ui.push_id(format!("levels_item:{}", level.asset_path), |ui| {
                                if ui.button(&level.display_name).clicked() {
                                    *open_asset_path = Some(level.asset_path.clone());
                                }
                            });
                        }
                    }
                });
            });
        });

        columns[2].vertical(|ui| {
            ui.heading("EntityTypes");
            ui.add_space(8.0);
            if ui.button("Regenerate entity types").clicked() {
                // Start background sync if not already running
                if !sync_state.running.load(std::sync::atomic::Ordering::SeqCst) {
                    sync_state.running.store(true, std::sync::atomic::Ordering::SeqCst);
                    let running_flag = sync_state.running.clone();
                    let result_slot = sync_state.result.clone();
                    std::thread::spawn(move || {
                        let res = crate::io::sync_entity_types_with_sprites();
                        if let Ok(mut guard) = result_slot.lock() {
                            *guard = Some(res);
                        }
                        running_flag.store(false, std::sync::atomic::Ordering::SeqCst);
                    });
                }
            }

            // Show running indicator while sync is in progress
                if sync_state.running.load(std::sync::atomic::Ordering::SeqCst) {
                ui.horizontal(|ui| {
                    ui.label("Update running...");
                    ui.spinner();
                });
            }
            ui.add_space(8.0);

            let list_height = ui.available_height();
            ui.push_id("entity_types_scroll_area", |ui| {
                egui::ScrollArea::vertical().max_height(list_height).show(ui, |ui| {
                    if let Some(error) = &entity_type_error {
                        ui.colored_label(egui::Color32::RED, error);
                    } else if entity_type_files.is_empty() {
                        ui.label("No entity-type JSON files found.");
                    } else {
                        for file_name in entity_type_files {
                            ui.push_id(format!("entity_type:{}", file_name), |ui| {
                                if ui.button(file_name).clicked() {
                                    let key = file_name.trim_end_matches(".json").to_string();
                                    selected = Some(key);
                                }
                            });
                        }
                    }
                });
            });
        });
    });

    selected
}

pub fn run() {
    crate::editor::run();
}

