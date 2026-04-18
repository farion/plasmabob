use bevy::prelude::*;

use crate::game::components::Blocking;
use crate::game::runtime_components::PreviousTransform;

pub fn track_previous_transform_system(
    mut commands: Commands,
    mut blockers: Query<(Entity, &Transform, Option<&mut PreviousTransform>), With<Blocking>>,
) {
    for (entity, transform, previous) in &mut blockers {
        let current_position = transform.translation.truncate();
        if let Some(mut previous) = previous {
            previous.position = current_position;
        } else {
            commands
                .entity(entity)
                .insert(PreviousTransform::from_translation(transform.translation));
        }
    }
}

