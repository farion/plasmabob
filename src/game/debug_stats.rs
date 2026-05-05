use bevy::prelude::*;

/// Lightweight debug-only statistics visible in the GameView debug HUD.
#[derive(Resource, Debug, Default, Clone)]
pub struct DebugStats {
    /// How many shape-cast calls were issued for projectiles
    pub projectile_shape_hits_calls: u32,
    /// How many candidate hits were returned/checked from those casts
    pub projectile_shape_hit_candidates: u32,
    /// Current frame rate (frames per second) sampled each frame
    pub fps: f32,
    /// Whether FPS line is shown in the debug HUD
    pub show_fps: bool,
    /// Whether counters (shape-casts/candidates) are shown
    pub show_counters: bool,
}

impl DebugStats {
    pub fn reset(&mut self) {
        self.projectile_shape_hits_calls = 0;
        self.projectile_shape_hit_candidates = 0;
        self.fps = 0.0;
        // Default to off — user toggles each HUD separately
        self.show_fps = false;
        self.show_counters = false;
    }
}
