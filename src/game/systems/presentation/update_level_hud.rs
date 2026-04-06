use crate::game::systems::presentation::types::{
    LevelKillsText, LevelTimeText, PlayerHealthPercentText, PlayerPlasmaCooldownPercentText,
};
use bevy::prelude::*;

pub(crate) fn update_level_hud(
    stats: Res<crate::LevelStats>,
    cached_level: Option<Res<crate::level::CachedLevelDefinition>>,
    mut kills_query: Query<
        &mut Text,
        (
            With<LevelKillsText>,
            Without<LevelTimeText>,
            Without<PlayerHealthPercentText>,
            Without<PlayerPlasmaCooldownPercentText>,
        ),
    >,
) {
    // Determine total enemies from cached level if present
    let mut total_enemies: u32 = 0;
    if let Some(cached) = cached_level {
        if let Ok(level_def) = cached.level_definition() {
            for ent in &level_def.entities {
                if let Some(entity_type_def) = level_def.entity_types.get(&ent.entity_type) {
                    if entity_type_def.components.iter().any(|c| c == "hostile") {
                        total_enemies += 1;
                    }
                }
            }
        }
    }

    if let Ok(mut text) = kills_query.single_mut() {
        // Show remaining enemies (total - killed) followed by total
        let remaining = total_enemies.saturating_sub(stats.enemies_killed);
        text.0 = format!("{}/{}", remaining, total_enemies);
    }
}
