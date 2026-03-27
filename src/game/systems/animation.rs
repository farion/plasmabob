use bevy::prelude::*;

use crate::game::components::animation::{
    AnimationPlayback, AnimationState, EntityState, HitStateTimer, PreloadedAnimations,
};
use crate::game::components::health::Health;
use crate::game::components::{AnimationCatalog, SpawnedLevelEntity};

pub(super) fn sync_death_state_from_health(mut entities: Query<(&Health, &mut AnimationState)>) {
    for (health, mut state) in &mut entities {
        if health.is_dead() {
            state.set(EntityState::Die);
        }
    }
}

pub(super) fn tick_hit_state_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut AnimationState, &mut HitStateTimer)>,
) {
    for (entity, mut state, mut hit_timer) in &mut entities {
        hit_timer.timer.tick(time.delta());
        if !hit_timer.timer.finished() {
            continue;
        }

        if state.current == EntityState::Hit && state.version == hit_timer.applied_at_state_version {
            state.set(EntityState::Default);
        }

        commands.entity(entity).remove::<HitStateTimer>();
    }
}

pub(super) fn apply_state_animation(
    images: Res<Assets<Image>>,
    time: Res<Time>,
    mut entities: Query<
        (
            &AnimationState,
            &AnimationCatalog,
            &PreloadedAnimations,
            &mut AnimationPlayback,
            &mut Sprite,
        ),
        With<SpawnedLevelEntity>,
    >,
) {
    for (state, animations, preloaded, mut playback, mut sprite) in &mut entities {
        let Some(frames) = animation_frames_for_state(animations, preloaded, state.current) else {
            continue;
        };

        advance_animation_playback(
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

fn animation_frames_for_state<'a>(
    catalog: &'a AnimationCatalog,
    preloaded: &'a PreloadedAnimations,
    state: EntityState,
) -> Option<&'a [Handle<Image>]> {
    if catalog.0.contains_key(state.animation_key()) {
        return preloaded
            .0
            .get(state.animation_key())
            .filter(|frames| !frames.is_empty())
            .map(Vec::as_slice);
    }

    preloaded
        .0
        .get(EntityState::Default.animation_key())
        .filter(|frames| !frames.is_empty())
        .map(Vec::as_slice)
}

fn advance_animation_playback(
    playback: &mut AnimationPlayback,
    state_version: u64,
    frame_count: usize,
    delta_secs: f32,
) {
    if frame_count == 0 {
        playback.frame_index = 0;
        playback.frame_elapsed = 0.0;
        return;
    }

    if playback.state_version != state_version {
        playback.state_version = state_version;
        playback.frame_index = 0;
        playback.frame_elapsed = 0.0;
    }

    if frame_count == 1 {
        playback.frame_index = 0;
        playback.frame_elapsed = 0.0;
        return;
    }

    playback.frame_elapsed += delta_secs.max(0.0);
    let frame_steps = (playback.frame_elapsed / playback.frame_duration_secs) as usize;
    if frame_steps == 0 {
        return;
    }

    playback.frame_elapsed -= frame_steps as f32 * playback.frame_duration_secs;
    playback.frame_index = (playback.frame_index + frame_steps) % frame_count;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_frames_at_configured_interval() {
        let mut playback = AnimationPlayback::new(0.2);

        advance_animation_playback(&mut playback, 0, 3, 0.19);
        assert_eq!(playback.frame_index, 0);

        advance_animation_playback(&mut playback, 0, 3, 0.01);
        assert_eq!(playback.frame_index, 1);

        advance_animation_playback(&mut playback, 0, 3, 0.4);
        assert_eq!(playback.frame_index, 0);
    }

    #[test]
    fn resets_to_first_frame_when_state_changes() {
        let mut playback = AnimationPlayback::new(0.5);
        playback.state_version = 1;
        playback.frame_index = 2;
        playback.frame_elapsed = 0.25;

        advance_animation_playback(&mut playback, 2, 4, 0.0);

        assert_eq!(playback.state_version, 2);
        assert_eq!(playback.frame_index, 0);
        assert_eq!(playback.frame_elapsed, 0.0);
    }
}
