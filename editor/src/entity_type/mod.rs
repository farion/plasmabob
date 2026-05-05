mod components_sidebar;
mod helpers;
mod hitbox;

pub use view::entity_type_view_ui;
mod view;
// Shared constants moved to module root so sibling submodules can access them via `super::`.
pub(crate) const HITBOX_EDGE_PICK_TOLERANCE_PX: f32 = 12.0;
pub(crate) const HITBOX_MIN_SIZE_PX: f32 = 1.0;
pub(crate) const PREVIEW_CANVAS_WIDTH_PX: f32 = 512.0;
pub(crate) const PREVIEW_CANVAS_HEIGHT_PX: f32 = 256.0;

// Re-export commonly used helpers/types so sibling modules and other files can
// reference them via `crate::entity_type::...`.
pub(crate) use helpers::{
    apply_to_staged_entity_type, cloned_staged_entity_type, component_default_value,
    component_object_snapshot, save_staged_entity_type, sorted_attribute_rows, AttributeUiRow,
};
pub(crate) use hitbox::EntityTypeEditorState;

// The generated component metadata is produced into OUT_DIR by build.rs and
// must be included at the module root so sibling submodules (helpers, view,
// etc.) can call `crate::entity_type::component_attribute_type(...)` and
// `crate::entity_type::component_declared_attributes(...)`.
include!(concat!(env!("OUT_DIR"), "/component_attr_map.rs"));
