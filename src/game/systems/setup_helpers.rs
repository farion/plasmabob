use bevy::prelude::*;

pub(crate) fn resolve_entity_z_index(
    entity_definition: &crate::game::level::EntityDefinition,
    entity_type: &crate::game::level::EntityTypeDefinition,
    is_player: bool,
) -> f32 {
    entity_definition.z_index.unwrap_or_else(|| {
        if is_player {
            20.0
        } else if entity_type.components.iter().any(|c| c == "npc") {
            10.0
        } else {
            0.0
        }
    })
}

