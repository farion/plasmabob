use bevy::audio::AudioSource;
use bevy::prelude::*;

// Shared types and constants that form the internal crate API between the
// `game_view` module and the `game::systems` modules. Keep this minimal and
// explicit so module boundaries are clear.

pub(crate) const PLAYER_MOVE_SPEED: f32 = 320.0;
pub(crate) const PLAYER_JUMP_SPEED: f32 = 900.0;
pub(crate) const MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN: f32 = 500.0;
pub(crate) const LEVEL_BOUNDARY_THICKNESS: f32 = 64.0;
pub(crate) const PLAYER_SCREEN_X_ANCHOR: f32 = 0.4;
pub(crate) const PLAYER_INVINCIBILITY_SECONDS: f32 = 1.0;

#[derive(Component)]
pub(crate) struct GameViewEntity;

#[derive(Component)]
pub(crate) struct DebugOverlayRoot;

#[derive(Component)]
pub(crate) struct TerrainBackgroundConfig {
    pub(crate) image: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct TerrainBackgroundReady;

#[derive(Component, Default)]
pub(crate) struct Grounded;

/// Attached to a `Text2d` entity that displays stats above a level entity in debug mode.
#[derive(Component)]
pub(crate) struct DebugStatsLabel {
    pub(crate) target: Entity,
}

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ActiveLevelBounds {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
    pub(crate) top: f32,
}

impl ActiveLevelBounds {
    pub(crate) fn from_window_and_level_size(window_size: Vec2, level_size: Vec2) -> Self {
        let left = -(window_size.x * 0.5);
        let bottom = -(window_size.y * 0.5);

        Self {
            left,
            right: left + level_size.x,
            bottom,
            top: bottom + level_size.y,
        }
    }

    pub(crate) fn center_x(self) -> f32 {
        (self.left + self.right) * 0.5
    }
}

#[derive(Resource, Default, Clone)]
pub(crate) struct LevelQuotes {
    pub(crate) clips: Vec<Handle<AudioSource>>,
}

#[derive(Resource, Clone)]
pub(crate) struct CombatSoundEffects {
    pub(crate) plasma_shot: Handle<AudioSource>,
    pub(crate) plasma_hit: Handle<AudioSource>,
    pub(crate) cockroach_die: Handle<AudioSource>,
}

#[derive(Resource)]
pub(crate) struct QuoteCooldown(pub(crate) Timer);

impl Default for QuoteCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(8.0, TimerMode::Once);
        // Start already finished so the first quote can play immediately.
        timer.tick(std::time::Duration::from_secs(8));
        Self(timer)
    }
}
