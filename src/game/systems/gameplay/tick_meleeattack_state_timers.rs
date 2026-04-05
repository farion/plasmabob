use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, MeleeAttackStateTimer};

pub(crate) fn tick_melee_attack_state_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut AnimationState, &mut MeleeAttackStateTimer)>,
) {
    for (entity, mut state, mut melee_timer) in &mut entities {
        melee_timer.timer.tick(time.delta());
        if melee_timer.timer.just_finished() {
            if state.current == crate::game::components::animation::EntityState::MeleeAttack {
                state.set(crate::game::components::animation::EntityState::Default);
            }
            commands.entity(entity).remove::<MeleeAttackStateTimer>();
        }
    }
}

