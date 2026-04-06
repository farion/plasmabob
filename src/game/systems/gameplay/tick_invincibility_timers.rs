use bevy::prelude::*;

use crate::game::components::health::InvincibilityTimer;

pub(crate) fn tick_invincibility_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut InvincibilityTimer)>,
) {
    for (entity, mut timer) in &mut timers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).remove::<InvincibilityTimer>();
        }
    }
}
