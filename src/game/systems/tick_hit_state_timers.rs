use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, HitStateTimer};

pub(crate) fn tick_hit_state_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut AnimationState, &mut HitStateTimer)>,
) {
    for (entity, mut state, mut hit_timer) in &mut entities {
        hit_timer.timer.tick(time.delta());
        if !hit_timer.timer.just_finished() {
            continue;
        }

        if state.current == crate::game::components::animation::EntityState::Hit
            && state.version == hit_timer.applied_at_state_version
        {
            state.set(crate::game::components::animation::EntityState::Default);
        }

        commands.entity(entity).remove::<HitStateTimer>();
    }
}

