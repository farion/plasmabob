use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::components::health::Health;
use crate::game::components::hostile::Hostile;
use crate::game::components::npc::Npc;
use crate::game::systems::gameplay::types::DeathQuotePlayed;
use crate::game::systems::systems_api::{GameViewEntity, LevelQuotes};
use crate::helper::audio_settings::AudioSettings;

pub(crate) fn play_hostile_death_quotes(
    mut commands: Commands,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    quotes: Option<Res<LevelQuotes>>,
    cooldown: Option<ResMut<crate::game::systems::systems_api::QuoteCooldown>>,
    dead_hostiles: Query<
        (Entity, &Health),
        (
            With<Hostile>,
            With<Npc>,
            With<SpawnedLevelEntity>,
            Without<DeathQuotePlayed>,
        ),
    >,
) {
    let Some(quotes) = quotes else {
        return;
    };

    if quotes.clips.is_empty() {
        return;
    }

    let Some(mut cooldown) = cooldown else {
        return;
    };

    cooldown.0.tick(time.delta());

    for (entity, health) in &dead_hostiles {
        if !health.is_dead() {
            continue;
        }
        commands.entity(entity).insert(DeathQuotePlayed);

        if !cooldown.0.just_finished() {
            continue;
        }

        let random_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0)
            .wrapping_add(entity.index_u32() as usize);
        let index = random_seed % quotes.clips.len();
        let quote_handle = quotes.clips[index].clone();

        commands.spawn((
            AudioPlayer::new(quote_handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(audio_settings.quotes_volume),
                ..default()
            },
            GameViewEntity,
        ));

        cooldown.0.reset();
    }
}
