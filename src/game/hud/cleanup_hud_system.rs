use bevy::prelude::*;

use crate::game::hud::components::HudRoot;

pub fn cleanup_hud_system(
    mut commands: Commands,
    hud_roots: Query<Entity, (With<HudRoot>, Without<ChildOf>)>,
) {
    for root in &hud_roots {
        commands.entity(root).try_despawn();
    }
}

