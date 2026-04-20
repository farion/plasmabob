// Minimal editor module surface kept during migration.
// The large editor.rs was split: level-specific code lives in crate::level
// (editor/src/level/*). Keep small re-exports here to avoid touching many
// call sites at once. The heavy lifting lives in crate::level now.

pub mod core;

// Re-export selected level state types so existing modules referencing
// `crate::editor::...` continue to work during the migration.
pub use crate::level::state::{
    CameraFitRequested, ClipboardEntity, ComponentAttributeDefinition, ComponentValueMapping,
    EditorDocument, EditorUiState, EntityTypeViewState, EntityTypesSyncState, HitboxOverlayState,
    LevelCatalog, PointerState, SceneDirty, SelectionState, ToastState, UndoCaptureState,
    UndoHistory, ZOverlayMode,
};

// Re-export moved level runtime items so existing code that refers to
// `crate::editor::run` and types like `ActiveCharacter` continue to work.
pub use crate::level::run::{
    run, ActiveCharacter, BackgroundTilesReady, EditorCamera, EditorMode, PendingBackgroundTiles,
    RenderedLevelEntity, RenderedZOverlay, SceneEntity,
};
