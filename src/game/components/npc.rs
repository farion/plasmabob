use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Npc;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Npc);
}
