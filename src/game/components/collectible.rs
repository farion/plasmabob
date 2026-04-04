use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Collectible;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Collectible);
}

