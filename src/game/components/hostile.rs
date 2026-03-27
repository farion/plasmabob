use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Hostile;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Hostile);
}


