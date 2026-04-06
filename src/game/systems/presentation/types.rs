use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayerHealthBarFill;

#[derive(Component)]
pub(crate) struct PlayerHealthPercentText;

#[derive(Component)]
pub(crate) struct PlayerPlasmaCooldownBarFill;

#[derive(Component)]
pub(crate) struct PlayerPlasmaCooldownPercentText;

#[derive(Component)]
pub(crate) struct LevelTimeText;

#[derive(Component)]
pub(crate) struct LevelKillsText;

#[derive(Resource)]
pub(crate) struct LevelTimer(pub Timer);

impl Default for LevelTimer {
    fn default() -> Self {
        LevelTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Component)]
pub(crate) struct BackgroundParallax;

#[derive(Component)]
pub(crate) struct ParallaxAnchor {
    pub(crate) base_x: f32,
    pub(crate) speed: f32,
}
