use bevy::prelude::*;

/// Runtime component that marks a floating damage/heal text and stores its animation state.
#[derive(Component, Debug)]
pub struct DamagePopup {
    /// Velocity in world units per second.
    pub velocity: Vec3,
    /// Angular velocity in radians per second applied to the text's Z rotation.
    pub angular_velocity: f32,
    /// Lifetime timer — when finished the entity will be despawned.
    pub life: Timer,
    /// True for heal (green), false for damage (red).
    pub is_heal: bool,
}

/// Tunable settings for damage popups.
#[derive(Resource)]
pub struct DamagePopupSettings {
    pub lifetime_secs: f32,
    pub upward_speed: f32,
    pub horizontal_spread: f32,
    pub base_font_size: f32,
    pub controlled_scale: f32,
}

impl Default for DamagePopupSettings {
    fn default() -> Self {
        Self {
            lifetime_secs: 0.9,
            upward_speed: 140.0,
            horizontal_spread: 40.0,
            // Increased base font size to make popups larger by default
            base_font_size: 32.0,
            // Slightly larger scale for controlled entities (player)
            controlled_scale: 1.7,
        }
    }
}



