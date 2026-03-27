use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer};
use crate::game::components::health::Health;
use crate::game::components::AnimationCatalog;

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
    asset_server: Res<AssetServer>,
    mut entities: Query<(&AnimationState, &AnimationCatalog, &mut Sprite), Changed<AnimationState>>,
) {
    for (state, animations, mut sprite) in &mut entities {
        let Some(path) = animation_path_for_state(animations, state.current) else {
            continue;
        };

        sprite.image = asset_server.load(path.to_string());
    }
}

fn animation_path_for_state<'a>(
    catalog: &'a AnimationCatalog,
    state: EntityState,
) -> Option<&'a str> {
    catalog
        .0
        .get(state.animation_key())
        .and_then(|frames| frames.first())
        .filter(|path| !path.is_empty())
        .map(String::as_str)
        .or_else(|| {
            catalog
                .0
                .get(EntityState::Default.animation_key())
                .and_then(|frames| frames.first())
                .filter(|path| !path.is_empty())
                .map(String::as_str)
        })
}
