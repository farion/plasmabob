use bevy::prelude::*;

/// Component for collectibles that heal the player when collected.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct EffectHeal(pub(crate) i32);

pub(crate) fn insert(entity: &mut EntityCommands, amount: i32) {
    entity.insert(EffectHeal(amount));
}

