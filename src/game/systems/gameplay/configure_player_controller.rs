use bevy::prelude::*;
use bevy::math::Dir2;
use avian2d::prelude::ShapeCaster;

use crate::game::components::player::Player;
use crate::game::components::hitbox::PrecomputedPlayerHitbox;

pub(crate) fn configure_player_controller(
    mut commands: Commands,
    mut players: Query<(Entity, &PrecomputedPlayerHitbox), (With<Player>, Without<ShapeCaster>)>,
) {
    for (player, precomputed_hitbox) in &mut players {
        let ground_collider: avian2d::prelude::Collider = precomputed_hitbox.ground_caster(false);
        commands.entity(player).insert(
            ShapeCaster::new(
                ground_collider,
                Vec2::ZERO,
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(8.0),
        );
    }
}

