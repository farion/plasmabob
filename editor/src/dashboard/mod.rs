// Moved from editor/src/dashboard.rs
// Minimal dashboard entry module used by `editor/src/main.rs`.
// It simply delegates to the editor run entrypoint so the editor starts in
// the same way. The actual render function for the level picker lives in
// `editor::dashboard` (module `src/editor/dashboard.rs`) where it can access
// editor-local types.

pub mod level_picker;
pub use level_picker::render_level_picker_columns;

pub mod run;
pub use run::run;
