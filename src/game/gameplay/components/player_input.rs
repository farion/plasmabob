use bevy::prelude::Component;

/// Stores the currently active input direction for a player-controlled entity.
/// Systems read this component to drive movement and actions.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct PlayerInput {
    /// Horizontal axis: -1.0 = left, 0.0 = none, 1.0 = right.
    pub horizontal: f32,
    /// Whether the jump button is held this frame.
    pub jump: bool,
    /// Whether the dash button was just pressed.
    pub dash: bool,
    /// Whether the primary attack button was just pressed.
    pub attack: bool,
}

