use bevy::prelude::*;

/// Mutable HUD data that is rendered each frame by HUD systems.
#[derive(Resource, Debug, Clone)]
pub struct HudState {
    pub health_frac: f32,
    pub plasma_cooldown_frac: f32,
    pub ego_frac: f32,
    pub level_seconds: f32,
    pub score: u64,
    pub lives: u8,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            health_frac: 1.0,
            plasma_cooldown_frac: 1.0,
            ego_frac: 0.0,
            level_seconds: 0.0,
            score: 0,
            lives: 3,
        }
    }
}

