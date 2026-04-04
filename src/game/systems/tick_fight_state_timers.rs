use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, FightStateTimer};

pub(crate) fn tick_fight_state_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut AnimationState, &mut FightStateTimer)>,
) {
    for (entity, mut state, mut fight_timer) in &mut entities {
        fight_timer.timer.tick(time.delta());
        if fight_timer.timer.just_finished() {
            if state.current == crate::game::components::animation::EntityState::Fight {
                state.set(crate::game::components::animation::EntityState::Default);
            }
            commands.entity(entity).remove::<FightStateTimer>();
        }
    }
}

