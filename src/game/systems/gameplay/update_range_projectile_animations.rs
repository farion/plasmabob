use bevy::prelude::*;

use crate::game::components::animation::{AnimationPlayback, PreloadedAnimations};
use crate::game::components::{AnimationFrameDurations};
use crate::game::systems::gameplay::types::RangeProjectileVisual;

pub(crate) fn update_range_projectile_animations(
    images: Res<Assets<Image>>,
    time: Res<Time>,
    mut query: Query<
        (
            &PreloadedAnimations,
            &AnimationFrameDurations,
            &mut AnimationPlayback,
            &mut Sprite,
        ),
        With<RangeProjectileVisual>,
    >,
) {
    let delta = time.delta_secs();

    for (preloaded, frame_durations, mut playback, mut sprite) in &mut query {
        let Some(frames) = preloaded.0.get("default") else {
            continue;
        };
        if frames.is_empty() {
            continue;
        }

        // Sync frame duration from the catalog.
        playback.frame_duration_secs = frame_durations
            .0
            .get("default")
            .copied()
            .unwrap_or(playback.frame_duration_secs)
            .max(0.001);

        // Advance playback.
        let frame_count = frames.len();
        if frame_count > 1 {
            playback.frame_elapsed += delta.max(0.0);
            let steps = (playback.frame_elapsed / playback.frame_duration_secs) as usize;
            if steps > 0 {
                playback.frame_elapsed -= steps as f32 * playback.frame_duration_secs;
                playback.frame_index = (playback.frame_index + steps) % frame_count;
            }
        }

        let Some(next) = frames.get(playback.frame_index) else {
            continue;
        };
        if sprite.image != *next && images.get(next).is_some() {
            sprite.image = next.clone();
        }
    }
}

