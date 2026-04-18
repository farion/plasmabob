use bevy::prelude::Component;

/// Movement component for player-controlled characters.
#[derive(Component, Debug, Clone, Copy)]
pub struct ControlledMovement {
    /// Horizontal move speed in virtual units per second.
    pub speed: f32,
    /// Jump impulse strength.
    pub jump_force: f32,
    /// Whether double jump is allowed.
    pub allow_double_jump: bool,
    /// Number of jumps performed since last grounded.
    pub jumps_performed: u8,
    /// Dash impulse strength.
    pub dash_force: f32,
    /// Maximum horizontal speed clamp (optional, 0 = no clamp).
    pub max_speed: f32,
    /// Optional facing direction: -1.0 = left, 1.0 = right
    pub facing: f32,
}

impl Default for ControlledMovement {
    fn default() -> Self {
        ControlledMovement {
            speed: 120.0,
            jump_force: 260.0,
            allow_double_jump: true,
            jumps_performed: 0,
            dash_force: 300.0,
            max_speed: 0.0,
            facing: 1.0,
        }
    }
}

// Use the macro to implement override_from_config for the component fields.
crate::impl_override_from_config!(ControlledMovement, crate::game::level::configs::ControlledMovementConfig,
    pick_f32 => [speed, jump_force, dash_force, max_speed, facing],
    pick_bool => [allow_double_jump],
    pick_u8 => [jumps_performed],
);

