use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, RangeAttackStateTimer};

pub(crate) fn tick_range_attack_state_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut AnimationState, &mut RangeAttackStateTimer)>,
) {
    for (entity, mut state, mut range_timer) in &mut entities {
        range_timer.timer.tick(time.delta());
        if range_timer.timer.just_finished() {
            if state.current == crate::game::components::animation::EntityState::RangeAttack {
                state.set(crate::game::components::animation::EntityState::Default);
            }
            commands.entity(entity).remove::<RangeAttackStateTimer>();
        }
    }
}

