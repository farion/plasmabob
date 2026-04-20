use bevy_egui::egui;
use serde_json::Value;

// State for the array editor modal. Stores a working copy (values) and the
// original snapshot so we can revert changes. Supports scalar arrays and
// one-level nested arrays (e.g. Vec<[f32;2]>). Inner arrays are edited as
// comma-separated strings in the UI and validated on commit.
#[derive(Clone)]
pub(crate) struct ArrayEditorState {
    pub component_name: String,
    pub attr_name: String,
    pub display_type: String,
    pub values: Vec<Value>,
    pub original: Vec<Value>,
    // Element-level flags
    pub element_is_array: bool,
    pub element_is_number: bool,
    pub inner_fixed_len: Option<usize>,
    pub outer_fixed_len: Option<usize>,
    // Parallel editable strings for inner-array textfields (one per outer element)
    pub inner_edit_strings: Vec<String>,
    // Optional modal position (for user to drag the modal). Stored as offset from centered position.
    pub modal_pos: egui::Pos2,
    // Track the modal size so we can start with a default height (300)
    // and persist manual resizes by the user. This prevents the modal
    // from automatically growing/shrinking when list contents change.
    pub modal_size: egui::Vec2,
    // Whether we've initialized the window's default size. We only call
    // Window::default_size on the first frame the modal is shown so that
    // egui will remember subsequent manual resizes instead of us forcing
    // a size each frame.
    pub modal_initialized: bool,
    // Unique per-open window id suffix so each modal session starts from
    // configured defaults instead of potentially stale egui window memory.
    pub window_session_id: u64,
}

// Helper parsed type info
pub(crate) struct ParsedArrayType {
    pub element_is_array: bool,
    pub element_is_number: bool,
    pub inner_fixed_len: Option<usize>,
    pub outer_fixed_len: Option<usize>,
}

pub(crate) fn parse_array_type_signature(type_str: &str) -> ParsedArrayType {
    // Try to detect patterns like "[f32;2]", "Vec<String>", "Vec<[f32;2]>",
    // or "array<number;2>". This is tolerant but extracts fixed lengths when
    // present.
    let s = type_str.replace(' ', "").replace('\t', "").to_string();
    // outer fixed: [T;N]
    if let Some(start) = s.find('[') {
        if let Some(semi) = s.find(';') {
            if let Some(end) = s.find(']') {
                // pattern [T;N]
                let after_semi = &s[semi + 1..end];
                if let Ok(n) = after_semi.parse::<usize>() {
                    // element type is between [ and ;
                    let elem = &s[start + 1..semi];
                    let element_is_array = elem.starts_with('[') || elem.starts_with("Vec<");
                    let element_is_number = elem.contains("f32")
                        || elem.contains("f64")
                        || elem.contains("number")
                        || elem.contains("i32")
                        || elem.contains("i64");
                    // if elem itself is [T;M], detect inner fixed len
                    let mut inner_fixed = None;
                    if elem.starts_with('[') {
                        if let Some(semi2) = elem.find(';') {
                            if let Some(end2) = elem.rfind(']') {
                                if let Ok(inner_n) = elem[semi2 + 1..end2].parse::<usize>() {
                                    inner_fixed = Some(inner_n);
                                }
                            }
                        }
                    }
                    return ParsedArrayType {
                        element_is_array,
                        element_is_number,
                        inner_fixed_len: inner_fixed,
                        outer_fixed_len: Some(n),
                    };
                }
            }
        }
    }

    // Vec<...> pattern
    if s.starts_with("Vec<") || s.starts_with("vec<") {
        if let Some(open) = s.find('<') {
            if let Some(close) = s.rfind('>') {
                let inner = &s[open + 1..close];
                // inner could be [T;N] or a primitive
                if inner.starts_with('[') {
                    // [T;N]
                    if let Some(semi) = inner.find(';') {
                        if let Some(end) = inner.find(']') {
                            if let Ok(n) = inner[semi + 1..end].parse::<usize>() {
                                let element_is_number = inner.contains("f32")
                                    || inner.contains("f64")
                                    || inner.contains("number")
                                    || inner.contains("i32")
                                    || inner.contains("i64");
                                return ParsedArrayType {
                                    element_is_array: true,
                                    element_is_number,
                                    inner_fixed_len: Some(n),
                                    outer_fixed_len: None,
                                };
                            }
                        }
                    }
                } else {
                    let element_is_number = inner.contains("f32")
                        || inner.contains("f64")
                        || inner.contains("number")
                        || inner.contains("i32")
                        || inner.contains("i64");
                    return ParsedArrayType {
                        element_is_array: false,
                        element_is_number,
                        inner_fixed_len: None,
                        outer_fixed_len: None,
                    };
                }
            }
        }
    }

    // array<number;N> style
    if s.starts_with("array<") {
        if let Some(open) = s.find('<') {
            if let Some(close) = s.rfind('>') {
                let inner = &s[open + 1..close];
                if let Some(semi) = inner.find(';') {
                    if let Ok(n) = inner[semi + 1..].parse::<usize>() {
                        let element_is_number = inner.contains("number") || inner.contains("f32");
                        return ParsedArrayType {
                            element_is_array: false,
                            element_is_number,
                            inner_fixed_len: None,
                            outer_fixed_len: Some(n),
                        };
                    }
                }
            }
        }
    }

    // Fallback: assume not array-of-array and not fixed length; detect numbers if present
    let element_is_number = s.contains("f32")
        || s.contains("f64")
        || s.contains("number")
        || s.contains("i32")
        || s.contains("i64");
    ParsedArrayType {
        element_is_array: false,
        element_is_number,
        inner_fixed_len: None,
        outer_fixed_len: None,
    }
}

pub(crate) fn inner_array_value_to_csv_string(v: &Value) -> String {
    if let Value::Array(arr) = v {
        let parts: Vec<String> = arr
            .iter()
            .map(|item| match item {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            })
            .collect();
        parts.join(",")
    } else {
        String::new()
    }
}

// Format a Value::Array into a short JSON-like string but print numbers using
// Rust's default formatting for f64 (which produces short, human-friendly
// decimals like "64.7" instead of long serialized floats).
pub(crate) fn format_array_short(arr: &Vec<Value>) -> String {
    let parts: Vec<String> = arr
        .iter()
        .map(|v| match v {
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    format!("{}", f)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => format!("\"{}\"", s),
            other => serde_json::to_string(other).unwrap_or_default(),
        })
        .collect();
    format!("[{}]", parts.join(","))
}

pub(crate) fn csv_string_to_value_array(
    s: &str,
    inner_is_number: bool,
) -> Result<Vec<Value>, String> {
    if s.trim().is_empty() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for token in s.split(',') {
        let t = token.trim();
        if inner_is_number {
            match t.parse::<f64>() {
                Ok(f) => {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        out.push(Value::Number(num));
                    } else {
                        return Err(format!("invalid number: '{}'", t));
                    }
                }
                Err(_) => return Err(format!("invalid number: '{}'", t)),
            }
        } else {
            out.push(Value::String(t.to_string()));
        }
    }
    Ok(out)
}

// Renders the modal editor for arrays. Because the caller (entity_types.rs)
// holds various borrows, this function does not mutate the staged entity type
// directly. Instead it returns an optional commit payload and a close flag.
// Return: (commit_values_opt, commit_target_opt (component, attr), commit_close_flag)
pub(crate) fn render_array_modal(
    ctx: &egui::Context,
    editor: &mut ArrayEditorState,
    toast: &mut crate::level::state::ToastState,
    time: &bevy::prelude::Time,
) -> (Option<Vec<Value>>, Option<(String, String)>, bool) {
    let mut commit_values: Option<Vec<Value>> = None;
    let mut commit_target: Option<(String, String)> = None;
    let mut commit_close = false;

    // Draw backdrop
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

    let mut wnd = egui::Window::new(format!("Edit {}", editor.display_type))
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
                    if !can_add {
                        toast.message = Some(
                            "Cannot add element: array is at fixed maximum length".to_string(),
                        );
                        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                    } else if editor.element_is_array {
                        if let Some(n) = editor.inner_fixed_len {
                            let mut v = Vec::new();
                            for _ in 0..n {
                                if editor.element_is_number {
                                    v.push(Value::Number(
                                        serde_json::Number::from_f64(0.0).unwrap(),
                                    ));
                                } else {
                                    v.push(Value::String(String::new()));
                                }
                            }
                            editor.values.push(Value::Array(v));
                            editor.inner_edit_strings.push(String::new());
                        } else {
                            editor.values.push(Value::Array(vec![]));
                            editor.inner_edit_strings.push(String::new());
                        }
                    } else {
                        if editor.element_is_number {
                            editor
                                .values
                                .push(Value::Number(serde_json::Number::from_f64(0.0).unwrap()));
                        } else {
                            editor.values.push(Value::String(String::new()));
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        commit_close = true;
                    }
                });
            });

            ui.separator();

            // Keep the list area explicitly bounded by the current modal height
            // so content cannot force the window to expand to parent height.
            let chrome_height = 120.0_f32;
            let list_max_height = (editor.modal_size.y - chrome_height).clamp(80.0, max_sz);
            egui::ScrollArea::vertical()
                .max_height(list_max_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                let mut remove_index: Option<usize> = None;
                let mut move_up: Option<usize> = None;
                let mut move_down: Option<usize> = None;
                // Ensure inner_edit_strings has an entry per outer element so
                // we can safely borrow a mutable reference to each string later.
                if editor.inner_edit_strings.len() < editor.values.len() {
                    let mut i = editor.inner_edit_strings.len();
                    while i < editor.values.len() {
                        let v = &editor.values[i];
                        if editor.element_is_array {
                            editor
                                .inner_edit_strings
                                .push(inner_array_value_to_csv_string(v));
                        } else {
                            editor.inner_edit_strings.push(match v {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                other => serde_json::to_string(other).unwrap_or_default(),
                            });
                        }
                        i += 1;
                    }
                }

                // Work on a snapshot to avoid simultaneous mutable borrows when
                // editing inner strings and applying value changes. Collect
                // pending updates and apply them after the UI iteration.
                let len = editor.values.len();
                let snapshots = editor.values.clone();
                let mut pending_updates: Vec<Option<Value>> = vec![None; len];

                for index in 0..len {
                    ui.horizontal(|ui| {
                        let snapshot = &snapshots[index];
                        if editor.element_is_array {
                            // inner array edited as CSV string
                            let s = editor.inner_edit_strings.get_mut(index).unwrap();
                            if ui.text_edit_singleline(s).changed() {
                                // validate
                                match csv_string_to_value_array(s, editor.element_is_number) {
                                    Ok(parsed) => {
                                        pending_updates[index] = Some(Value::Array(parsed));
                                    }
                                    Err(e) => {
                                        toast.message = Some(format!("Invalid inner array: {}", e));
                                        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
                                    }
                                }
                            }
                        } else if editor.element_is_number {
                            let mut f = snapshot.as_f64().unwrap_or(0.0);
                            if ui.add(egui::DragValue::new(&mut f).speed(1.0)).changed() {
                                if let Some(n) = serde_json::Number::from_f64(f) {
                                    pending_updates[index] = Some(Value::Number(n));
                                }
                            }
                        } else {
                            let mut s =
                                snapshot.as_str().map(|s| s.to_string()).unwrap_or_default();
                            if ui.text_edit_singleline(&mut s).changed() {
                                pending_updates[index] = Some(Value::String(s));
                            }
                        }

                        // Use phosphor icons for element controls. Disable up/down at
                        // list boundaries so users cannot move beyond the ends.
                        let up_enabled = index > 0;
                        let up_resp = ui.add_enabled(up_enabled, egui::Button::new(egui_phosphor_icons::icons::CARET_UP).min_size(egui::vec2(20.0, 20.0)));
                        if up_resp.clicked() && up_enabled {
                            move_up = Some(index);
                        }

                        let down_enabled = index + 1 < len;
                        let down_resp = ui.add_enabled(down_enabled, egui::Button::new(egui_phosphor_icons::icons::CARET_DOWN).min_size(egui::vec2(20.0, 20.0)));
                        if down_resp.clicked() && down_enabled {
                            move_down = Some(index);
                        }

                        let trash_resp = ui.add(egui::Button::new(egui_phosphor_icons::icons::TRASH).min_size(egui::vec2(20.0, 20.0)));
                        if trash_resp.clicked() {
                            remove_index = Some(index);
                        }
                    });
                }

                // Apply pending updates
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

            ui.horizontal(|ui| {
                // If the array has a fixed outer size, only allow commit when
                // that size is reached.
                let can_commit = match editor.outer_fixed_len {
                    Some(required) => editor.values.len() == required,
                    None => true,
                };
                if ui.add_enabled(can_commit, egui::Button::new("Apply")).clicked() {
                    commit_values = Some(editor.values.clone());
                    commit_target = Some((editor.component_name.clone(), editor.attr_name.clone()));
                    commit_close = true;
                }
                if ui.button("Revert").clicked() {
                    editor.values = editor.original.clone();
                    editor.inner_edit_strings.clear();
                    for v in &editor.values {
                        if editor.element_is_array {
                            editor
                                .inner_edit_strings
                                .push(inner_array_value_to_csv_string(v));
                        } else {
                            editor.inner_edit_strings.push(match v {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                other => serde_json::to_string(other).unwrap_or_default(),
                            });
                        }
                    }
                }
            });
        });
    });

    // Persist user-resized window size and drag offset for this open modal
    // session (clamped to configured bounds).
    if let Some(window_info) = res {
        let rect = window_info.response.rect;
        editor.modal_size = egui::vec2(
            rect.width().clamp(min_sz, max_sz),
            rect.height().clamp(min_sz, max_sz),
        );
        let rect_center = rect.center();
        editor.modal_pos = egui::pos2(rect_center.x - center.x, rect_center.y - center.y);
        editor.modal_initialized = true;
    }

    (commit_values, commit_target, commit_close)
}
