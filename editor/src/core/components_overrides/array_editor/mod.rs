mod format;
mod modal;
mod modal_helpers;
mod parse;

pub(crate) use format::{format_array_short, inner_array_value_to_csv_string};
pub(crate) use modal::render_array_modal;
pub(crate) use parse::parse_array_type_signature;

// Session id generator moved to core/ui.rs to be shared by multiple modules.

#[derive(Clone)]
pub(crate) struct ArrayEditorState {
    pub component_name: String,
    pub attr_name: String,
    pub display_type: String,
    pub values: Vec<serde_json::Value>,
    pub original: Vec<serde_json::Value>,
    pub element_is_array: bool,
    pub element_is_number: bool,
    pub inner_fixed_len: Option<usize>,
    pub outer_fixed_len: Option<usize>,
    pub inner_edit_strings: Vec<String>,
    pub modal_pos: bevy_egui::egui::Pos2,
    pub modal_size: bevy_egui::egui::Vec2,
    pub modal_initialized: bool,
    pub window_session_id: u64,
}

pub(crate) struct ParsedArrayType {
    pub element_is_array: bool,
    pub element_is_number: bool,
    pub inner_fixed_len: Option<usize>,
    pub outer_fixed_len: Option<usize>,
}
