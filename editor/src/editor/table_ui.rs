use bevy::prelude::*;
// table_ui contains shared constants/helpers for table widths. Demo removed.

// Minimal table demo resource: stable column widths for emulating colspan.
#[derive(Resource, Debug, Clone)]
pub(crate) struct ColumnWidths {
    pub widths: Vec<f32>,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        // Default placeholder; the entity-type UI will compute precise widths.
        Self { widths: vec![120.0; 6] }
    }
}

// Provide constants to align with the entity_types sidebar layout
pub const ATTR_NAME_COLUMN_WIDTH: f32 = 150.0;
pub const CLEAR_BUTTON_COLUMN_WIDTH: f32 = 64.0;

/// Compute the middle column width given the available width in the sidebar.
pub fn compute_middle_col_width(total_avail: f32, spacing_reserved: f32) -> f32 {
    (total_avail - ATTR_NAME_COLUMN_WIDTH - CLEAR_BUTTON_COLUMN_WIDTH - spacing_reserved).max(80.0)
}

// Demo code removed. TableBuilder helpers remain in this module for shared use.
