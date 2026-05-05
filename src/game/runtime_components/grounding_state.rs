use bevy::prelude::*;

/// Runtime grounding data collected during collision resolution.
#[derive(Component, Debug, Clone, Copy)]
pub struct GroundingState {
    /// Sum of upward support normals gathered from top contacts this frame.
    pub support_normal_sum_y: f32,
    /// Velocity inherited from the supporting platform this frame.
    pub support_velocity: Vec2,
    /// Supporting blocker entity from the last valid floor contact.
    pub support_entity: Option<Entity>,
    /// Time without valid support, used for grounded hysteresis.
    pub unsupported_time: f32,
}

impl GroundingState {
    pub fn clear_step_contacts(&mut self) {
        self.support_normal_sum_y = 0.0;
        self.support_velocity = Vec2::ZERO;
        self.support_entity = None;
    }
}

impl Default for GroundingState {
    fn default() -> Self {
        Self {
            support_normal_sum_y: 0.0,
            support_velocity: Vec2::ZERO,
            support_entity: None,
            unsupported_time: 0.0,
        }
    }
}

