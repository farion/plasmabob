use bevy::prelude::*;

/// Stores the previous frame world position for motion transfer calculations.
#[derive(Component, Debug, Clone, Copy)]
pub struct PreviousTransform {
    pub position: Vec2,
}

impl PreviousTransform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            position: translation.truncate(),
        }
    }
}

