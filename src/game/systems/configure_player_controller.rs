use bevy::prelude::*;
use bevy::math::Dir2;

use crate::game::components::player::Player;
use crate::game::components::hitbox::PrecomputedPlayerHitbox;

pub(crate) fn configure_player_controller(
    mut commands: Commands,
    mut players: Query<(Entity, &PrecomputedPlayerHitbox), (With<Player>, Without<bevy::prelude::ShapeCaster>)>,
) {
    for (player, precomputed_hitbox) in &mut players {
        commands.entity(player).insert(
            bevy::prelude::ShapeCaster::new(
                precomputed_hitbox.ground_caster(false),
                Vec2::ZERO,
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(8.0),
        );
    }
}

