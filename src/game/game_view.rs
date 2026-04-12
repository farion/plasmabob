use bevy::prelude::*;

pub struct GameViewPlugin;

// ...existing code...

impl Plugin for GameViewPlugin {
    fn build(&self, _app: &mut App) {
        // Minimal, non-panicking implementation for the Game view plugin.
        // The full project registers game systems and resources here; at
        // minimum avoid a panic so the application can start. Concrete
        // systems should be added in this function as the codebase grows.
        let _ = _app; // keep the unused variable quiet until systems are added
    }
}
