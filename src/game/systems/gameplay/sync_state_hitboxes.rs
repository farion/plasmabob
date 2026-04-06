use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::components::animation::{AnimationState, EntityState};
use crate::game::components::hitbox::{
    self, PolygonHitbox, PrecomputedPlayerHitbox, StateHitboxCatalog,
};

pub(crate) fn sync_state_hitboxes(
    mut entities: Query<
        (
            &AnimationState,
            &StateHitboxCatalog,
            &mut PolygonHitbox,
            Option<&mut PrecomputedPlayerHitbox>,
            Option<&mut Collider>,
        ),
        With<SpawnedLevelEntity>,
    >,
) {
    for (state, catalog, mut polygon_hitbox, precomputed_hitbox, collider) in &mut entities {
        let state_key = state.current.animation_key();
        let Some(next_hitbox) = hitbox_for_state(catalog, state_key) else {
            continue;
        };

        if polygon_hitbox.points == next_hitbox.points {
            continue;
        }

        polygon_hitbox.points = next_hitbox.points.clone();

        if let Some(mut precomputed) = precomputed_hitbox {
            *precomputed = PrecomputedPlayerHitbox::from_polygon_hitbox(&polygon_hitbox);
            continue;
        }

        if let Some(mut dynamic_collider) = collider {
            *dynamic_collider = hitbox::collider_from_points(polygon_hitbox.points.clone());
        }
    }
}

/// Return the polygon hitbox for the given state key, falling back to
/// `Default` if needed.
fn hitbox_for_state<'a>(
    catalog: &'a StateHitboxCatalog,
    state_key: &str,
) -> Option<&'a PolygonHitbox> {
    catalog
        .0
        .get(state_key)
        .or_else(|| catalog.0.get(EntityState::Default.animation_key()))
}
