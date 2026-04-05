use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState};
use crate::game::components::health::Health;

pub(crate) fn sync_death_state_from_health(mut entities: Query<(&Health, &mut AnimationState)>) {
    for (health, mut state) in &mut entities {
        if health.is_dead() {
            state.set(EntityState::Die);
        }
    }
}

