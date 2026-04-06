use bevy::prelude::*;

use crate::game::systems::presentation::types::{LevelTimeText, LevelTimer};

pub(crate) fn tick_level_time(
    time: Res<Time>,
    mut stats: ResMut<crate::LevelStats>,
    _cached_level: Option<Res<crate::level::CachedLevelDefinition>>,
    mut timer: ResMut<LevelTimer>,
    mut time_query: Query<&mut Text, With<LevelTimeText>>,
) {
    // Always accumulate elapsed time
    let delta = time.delta_secs();
    stats.total_time_seconds += delta;

    // Tick the 1s timer and only update the displayed text when it finishes
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let minutes = (stats.total_time_seconds as u32) / 60;
    let seconds = (stats.total_time_seconds as u32) % 60;
    let time_str = format!("{}:{:02}", minutes, seconds);

    if let Ok(mut text) = time_query.single_mut() {
        text.0 = time_str;
    }
}
