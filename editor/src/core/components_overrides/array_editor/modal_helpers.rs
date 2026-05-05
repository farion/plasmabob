use bevy_egui::egui;
use serde_json::Value;

pub(super) fn csv_string_to_value_array(
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

pub(super) fn add_element(
    editor: &mut super::ArrayEditorState,
    toast: &mut crate::level::state::ToastState,
    time: &bevy::prelude::Time,
) {
    let can_add = match editor.outer_fixed_len {
        Some(max) => editor.values.len() < max,
        None => true,
    };
    if !can_add {
        toast.message = Some("Cannot add element: array is at fixed maximum length".to_string());
        toast.expires_at_seconds = time.elapsed_secs_f64() + 3.0;
        return;
    }

    if editor.element_is_array {
        if let Some(n) = editor.inner_fixed_len {
            let mut v = Vec::new();
            for _ in 0..n {
                if editor.element_is_number {
                    v.push(Value::Number(serde_json::Number::from_f64(0.0).unwrap()));
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
    } else if editor.element_is_number {
        editor
            .values
            .push(Value::Number(serde_json::Number::from_f64(0.0).unwrap()));
    } else {
        editor.values.push(Value::String(String::new()));
    }
}

pub(super) fn ensure_inner_strings(editor: &mut super::ArrayEditorState) {
    if editor.inner_edit_strings.len() < editor.values.len() {
        let mut i = editor.inner_edit_strings.len();
        while i < editor.values.len() {
            let v = &editor.values[i];
            if editor.element_is_array {
                editor
                    .inner_edit_strings
                    .push(super::inner_array_value_to_csv_string(v));
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
}

pub(super) fn reset_values(editor: &mut super::ArrayEditorState) {
    editor.values = editor.original.clone();
    editor.inner_edit_strings.clear();
    for v in &editor.values {
        if editor.element_is_array {
            editor
                .inner_edit_strings
                .push(super::inner_array_value_to_csv_string(v));
        } else {
            editor.inner_edit_strings.push(match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                other => serde_json::to_string(other).unwrap_or_default(),
            });
        }
    }
}

pub(super) fn persist_window_state(
    editor: &mut super::ArrayEditorState,
    res: Option<egui::InnerResponse<Option<()>>>,
    center: egui::Pos2,
    min_sz: f32,
    max_sz: f32,
) {
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
}
