use bevy::prelude::{Component, Vec2};

/// Data-driven platform movement along authored world-space waypoints.
///
/// Waypoints are expected to include the start position as the first entry.
#[derive(Component, Debug, Clone)]
pub struct MovingPlatform {
    /// Path points in world coordinates (x, y).
    pub waypoints: Vec<Vec2>,
    /// Movement speed in virtual units per second.
    pub speed: f32,
    /// Whether the platform loops back to waypoint 0 after the last point.
    pub repeat: bool,
    /// Allows temporary runtime pause without removing the component.
    pub enabled: bool,
    /// Internal index of the current target waypoint.
    pub target_index: usize,
}

impl Default for MovingPlatform {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            speed: 0.0,
            repeat: false,
            enabled: true,
            target_index: 1,
        }
    }
}

impl MovingPlatform {
    /// Returns true when authored data allows movement.
    pub fn can_move(&self) -> bool {
        self.enabled && self.speed > 0.0 && self.waypoints.len() > 1
    }

    /// Advances to the next waypoint and returns whether movement should continue.
    pub fn advance_target(&mut self) -> bool {
        let count = self.waypoints.len();
        if count <= 1 {
            self.target_index = 0;
            return false;
        }

        if self.target_index + 1 < count {
            self.target_index += 1;
            return true;
        }

        if self.repeat {
            self.target_index = 0;
            return true;
        }

        self.target_index = count - 1;
        false
    }
}

// Use the macro to generate override_from_config and include a post-processing block
crate::impl_override_from_config!(MovingPlatform, crate::game::level::configs::MovingPlatformConfig,
    pick_f32 => [speed],
    pick_bool => [repeat, enabled],
    pick_waypoints => [waypoints],
);
