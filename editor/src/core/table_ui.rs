use bevy::prelude::*;
// table_ui contains shared constants/helpers for table widths.

// Minimal table demo resource: stable column widths for emulating colspan.
#[derive(Resource, Debug, Clone)]
pub struct ColumnWidths {
    pub widths: Vec<f32>,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        // Default placeholder; the entity-type UI will compute precise widths.
        Self {
            widths: vec![120.0; 6],
        }
    }
}
