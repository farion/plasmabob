use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::components::player::Player;
use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::animation::{AnimationState, HitStateTimer, EntityState, can_set_state};

pub(crate) fn maintain_player_fight_state(
    beams: Query<&PlasmaBeam>,
    mut players: Query<(Entity, &mut AnimationState, Option<&HitStateTimer>), With<Player>>,
) {
    let active_beam_owners: HashSet<Entity> = beams.iter().map(|beam| beam.player_entity).collect();

    for (player_entity, mut state, hit_timer) in &mut players {
        if !active_beam_owners.contains(&player_entity) {
            continue;
        }

        if can_set_state(&state, hit_timer, None, EntityState::Fight) {
            state.set(EntityState::Fight);
        }
    }
}

