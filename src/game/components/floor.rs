use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Floor;

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Floor);
}
