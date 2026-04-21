use bevy_egui::egui;
use serde_json::Value;

pub(crate) fn render_array_modal(
    ctx: &egui::Context,
    editor: &mut super::ArrayEditorState,
    toast: &mut crate::level::state::ToastState,
    time: &bevy::prelude::Time,
) -> (Option<Vec<Value>>, Option<(String, String)>, bool) {
    let mut commit_values: Option<Vec<Value>> = None;
    let mut commit_target: Option<(String, String)> = None;
    let mut commit_close = false;

    let layer_id = egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("array_editor_modal_layer"),
    );
    ctx.layer_painter(layer_id).rect_filled(
        ctx.available_rect(),
        0.0,
        egui::Color32::from_black_alpha(160),
    );

    let center = ctx.available_rect().center();

    let min_sz = 300.0_f32;
    let max_sz = 800.0_f32;
    editor.modal_size.x = editor.modal_size.x.clamp(min_sz, max_sz);
    editor.modal_size.y = editor.modal_size.y.clamp(min_sz, max_sz);

    let pos = egui::pos2(
        center.x + editor.modal_pos.x - editor.modal_size.x * 0.5,
        center.y + editor.modal_pos.y - editor.modal_size.y * 0.5,
    );

    let wnd = egui::Window::new(format!("Edit {}", editor.display_type))
        .collapsible(false)
        .default_pos([pos.x, pos.y])
        .default_size([editor.modal_size.x, editor.modal_size.y])
        .min_size([min_sz, min_sz])
        .max_size([max_sz, max_sz])
        .resizable(true);
    let wnd = wnd.id(egui::Id::new(format!(
        "array_editor_modal::{}::{}::{}",
        editor.component_name, editor.attr_name, editor.window_session_id
    )));

    let res = wnd.show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let can_add = match editor.outer_fixed_len {
                    Some(max) => editor.values.len() < max,
                    None => true,
                };

                if ui
                    .add_enabled(can_add, egui::Button::new(egui_phosphor_icons::icons::PLUS))
                    .clicked()
                {
                    super::modal_helpers::add_element(editor, toast, time);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(egui_phosphor_icons::icons::X)
                                .min_size(egui::vec2(20.0, 20.0)),
                        )
                        .clicked()
                    {
                        commit_close = true;
                    }
                });
            });

            ui.separator();

            let chrome_height = 120.0_f32;
            let list_max_height = (editor.modal_size.y - chrome_height).clamp(80.0, max_sz);
            egui::ScrollArea::vertical()
                .max_height(list_max_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut remove_index: Option<usize> = None;
                    let mut move_up: Option<usize> = None;
                    let mut move_down: Option<usize> = None;
                    super::modal_helpers::ensure_inner_strings(editor);

                    let len = editor.values.len();
                    let snapshots = editor.values.clone();
                    let mut pending_updates: Vec<Option<Value>> = vec![None; len];

                    for index in 0..len {
                        ui.horizontal(|ui| {
                            let snapshot = &snapshots[index];
                            if editor.element_is_array {
                                let s = editor.inner_edit_strings.get_mut(index).unwrap();
                                let frame_resp = egui::Frame::NONE
                                    .fill(egui::Color32::from_rgb(55, 60, 66))
                                    .show(ui, |ui| ui.text_edit_singleline(s));
                                if frame_resp.inner.changed() {
                                    match super::modal_helpers::csv_string_to_value_array(
                                        s,
                                        editor.element_is_number,
                                    ) {
                                        Ok(parsed) => {
                                            pending_updates[index] = Some(Value::Array(parsed));
                                        }
                                        Err(e) => {
                                            toast.message =
                                                Some(format!("Invalid inner array: {}", e));
                                            toast.expires_at_seconds =
                                                time.elapsed_secs_f64() + 3.0;
                                        }
                                    }
                                }
                            } else if editor.element_is_number {
                                let mut f = snapshot.as_f64().unwrap_or(0.0);
                                let frame_resp = egui::Frame::NONE
                                    .fill(egui::Color32::from_rgb(55, 60, 66))
                                    .show(ui, |ui| ui.add(egui::DragValue::new(&mut f).speed(1.0)));
                                if frame_resp.inner.changed() {
                                    if let Some(n) = serde_json::Number::from_f64(f) {
                                        pending_updates[index] = Some(Value::Number(n));
                                    }
                                }
                            } else {
                                let mut s =
                                    snapshot.as_str().map(|s| s.to_string()).unwrap_or_default();
                                let frame_resp = egui::Frame::NONE
                                    .fill(egui::Color32::from_rgb(55, 60, 66))
                                    .show(ui, |ui| ui.text_edit_singleline(&mut s));
                                if frame_resp.inner.changed() {
                                    pending_updates[index] = Some(Value::String(s));
                                }
                            }

                            let up_enabled = index > 0;
                            let up_resp = ui.add_enabled(
                                up_enabled,
                                egui::Button::new(egui_phosphor_icons::icons::CARET_UP)
                                    .min_size(egui::vec2(20.0, 20.0)),
                            );
                            if up_resp.clicked() && up_enabled {
                                move_up = Some(index);
                            }

                            let down_enabled = index + 1 < len;
                            let down_resp = ui.add_enabled(
                                down_enabled,
                                egui::Button::new(egui_phosphor_icons::icons::CARET_DOWN)
                                    .min_size(egui::vec2(20.0, 20.0)),
                            );
                            if down_resp.clicked() && down_enabled {
                                move_down = Some(index);
                            }

                            let trash_resp = ui.add(
                                egui::Button::new(egui_phosphor_icons::icons::TRASH)
                                    .min_size(egui::vec2(20.0, 20.0)),
                            );
                            if trash_resp.clicked() {
                                remove_index = Some(index);
                            }
                        });
                    }

                    for (i, upd) in pending_updates.into_iter().enumerate() {
                        if let Some(v) = upd {
                            editor.values[i] = v;
                        }
                    }

                    if let Some(idx) = remove_index {
                        editor.values.remove(idx);
                        editor.inner_edit_strings.remove(idx);
                    }
                    if let Some(idx) = move_up {
                        editor.values.swap(idx, idx - 1);
                        editor.inner_edit_strings.swap(idx, idx - 1);
                    }
                    if let Some(idx) = move_down {
                        editor.values.swap(idx, idx + 1);
                        editor.inner_edit_strings.swap(idx, idx + 1);
                    }
                });

            ui.separator();

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let can_commit = match editor.outer_fixed_len {
                    Some(required) => editor.values.len() == required,
                    None => true,
                };
                if ui
                    .add_enabled(
                        can_commit,
                        egui::Button::new(egui_phosphor_icons::icons::CHECK)
                            .min_size(egui::vec2(60.0, 24.0)),
                    )
                    .clicked()
                {
                    commit_values = Some(editor.values.clone());
                    commit_target = Some((editor.component_name.clone(), editor.attr_name.clone()));
                    commit_close = true;
                }

                if ui
                    .add(
                        egui::Button::new(egui_phosphor_icons::icons::ARROW_COUNTER_CLOCKWISE)
                            .min_size(egui::vec2(60.0, 24.0)),
                    )
                    .clicked()
                {
                    super::modal_helpers::reset_values(editor);
                }
            });
        });
    });

    super::modal_helpers::persist_window_state(editor, res, center, min_sz, max_sz);

    (commit_values, commit_target, commit_close)
}
