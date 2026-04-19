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

// Constants removed: column sizing is driven by the shared ColumnWidths
// resource and computed by each UI that owns the layout. Demo helpers
// and TableBuilder utilities remain in this module for shared use.

// Demo code removed. TableBuilder helpers remain in this module for shared use.
