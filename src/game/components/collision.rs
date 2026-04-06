use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Collision;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Collision);
}
