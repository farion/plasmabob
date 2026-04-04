use bevy::prelude::*;

use crate::game::components::hitbox::PrecomputedPlayerHitbox;
use crate::game::components::player::Player;
use crate::game::components::moving::Moving;

pub(crate) fn sync_player_hitbox_orientation(
    mut players: Query<
        (&Sprite, &PrecomputedPlayerHitbox, &mut avian2d::prelude::Collider, Option<&mut bevy::prelude::ShapeCaster>),
        (Or<(With<Player>, With<Moving>)>, Changed<Sprite>),
    >,
) {
    for (sprite, precomputed_hitbox, mut collider, shape_caster) in &mut players {
        *collider = precomputed_hitbox.collider(sprite.flip_x);

        if let Some(mut shape_caster) = shape_caster {
            shape_caster.shape = precomputed_hitbox.ground_caster(sprite.flip_x);
        }
    }
}

