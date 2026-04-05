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
                                // folder name (stem of json file) used as selection key
                                let folder_opt = std::path::Path::new(&world.asset_path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .map(|s| s.to_string());

                                let is_selected = match (&catalog.selected_world, &folder_opt) {
                                    (Some(sel), Some(folder)) => sel == folder,
                                    _ => false,
                                };

                                // Render a button inside a filled Frame so it always has a background.
                                // When selected, use a highlighted color; otherwise use the default
                                // widget background so it looks like a normal button.
                                // Render the world as a normal button so it has the same
                                // padding and interaction as other buttons (e.g. "Force Reread").
                                // If the world is selected, draw a highlighted outline around it.
                                // Capture the corner radius value before mutably borrowing `ui`.
                                let rounding = ui.visuals().widgets.inactive.corner_radius;
                                let btn = egui::Button::new(&world.display_name);
                                let resp = ui.add(btn);
                                if resp.clicked() {
                                    catalog.selected_world = folder_opt.clone();
                                }

                                if is_selected {
                                    // draw blue outline around the button rect
                                    let stroke_color = egui::Color32::from_rgb(60, 120, 200);
                                    let stroke = egui::Stroke::new(2.0, stroke_color);
                                    ui.painter().rect_stroke(resp.rect.expand(2.0), rounding, stroke, egui::StrokeKind::Inside);
                                }
                            });
                        }
                    }
                });
            });
        });

        columns[1].vertical(|ui| {
            // Show the selected world in parentheses after the Levels heading
            let selected_world = catalog.selected_world.as_deref();
            if let Some(folder) = selected_world {
                ui.heading(format!("Levels ({})", folder));
            } else {
                ui.heading("Levels");
            }
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

