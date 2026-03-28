use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Exit;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Exit);
}

