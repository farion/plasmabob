use bevy::prelude::*;
use std::time::Duration;

/// Drives sprite-sheet animation from an ordered list of asset-path frames.
/// A repeating timer advances the current frame at a fixed interval.
#[derive(Component, Debug, Clone)]
pub struct AnimationConfig {
    /// Ordered list of sprite asset paths that form the animation cycle.
    pub frames: Vec<String>,
    /// Index of the frame that is currently displayed.
    pub current_frame: usize,
    /// Timer that fires when it is time to advance to the next frame.
    pub frame_timer: Timer,
}

impl AnimationConfig {
    /// Create a new animation config for the given frames and per-frame duration.
    pub fn new(frames: Vec<String>, frame_ms: u64) -> Self {
        AnimationConfig {
            frames,
            current_frame: 0,
            frame_timer: Timer::new(Duration::from_millis(frame_ms), TimerMode::Repeating),
        }
    }

    /// Advance the animation by `delta`. Returns `true` when the frame index changed.
    pub fn tick(&mut self, delta: Duration) -> bool {
        self.frame_timer.tick(delta);
        if self.frame_timer.just_finished() && self.frames.len() > 1 {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            return true;
        }
        false
    }

    /// Return the asset path of the currently active frame, or `None` when empty.
    pub fn current_frame_path(&self) -> Option<&str> {
        self.frames.get(self.current_frame).map(|s| s.as_str())
    }
}

