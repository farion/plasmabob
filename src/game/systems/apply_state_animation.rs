use bevy::prelude::*;

use crate::game::components::animation::{AnimationPlayback, AnimationState, PreloadedAnimations};
use crate::game::components::{AnimationCatalog, AnimationFrameDurations, SpawnedLevelEntity};

// Use fully-qualified helper calls to avoid any module resolution ambiguity.

pub(crate) fn apply_state_animation(
    images: Res<Assets<Image>>,
    time: Res<Time>,
    mut entities: Query<
        (
            &AnimationState,
            &AnimationCatalog,
            &AnimationFrameDurations,
            &PreloadedAnimations,
            &mut AnimationPlayback,
            &mut Sprite,
        ),
        With<SpawnedLevelEntity>,
    >,
) {
    for (state, animations, frame_durations, preloaded, mut playback, mut sprite) in &mut entities {
        let Some(frames) = crate::game::systems::common::animation_helpers::animation_frames_for_state(animations, preloaded, state.current) else {
            continue;
        };

        playback.frame_duration_secs = crate::game::systems::common::animation_helpers::animation_frame_duration_for_state(frame_durations, state.current);

        crate::game::systems::common::animation_helpers::advance_animation_playback(
            &mut playback,
            state.version,
            frames.len(),
            time.delta_secs(),
        );

        let Some(next_image) = frames.get(playback.frame_index) else {
            continue;
        };

        if sprite.image == *next_image {
            continue;
        }

        // Keep the previous frame visible until the next image is fully loaded.
        if images.get(next_image).is_none() {
            continue;
        }

        sprite.image = next_image.clone();
    }
}


