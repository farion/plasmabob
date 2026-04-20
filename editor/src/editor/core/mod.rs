// Re-export the canonical ColumnWidths resource from crate::core so callers
// that still refer to `crate::editor::core` continue to work during the
// migration. The real implementation lives in `crate::core::table_ui`.
pub use crate::core::ColumnWidths;
