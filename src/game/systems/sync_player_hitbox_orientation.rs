use bevy::prelude::*;
use avian2d::prelude::ShapeCaster;

use crate::game::components::hitbox::PrecomputedPlayerHitbox;
use crate::game::components::player::Player;
use crate::game::components::moving::Moving;

pub(crate) fn sync_player_hitbox_orientation(
    mut players: Query<
        (&Sprite, &PrecomputedPlayerHitbox, &mut avian2d::prelude::Collider, Option<&mut ShapeCaster>),
        (Or<(With<Player>, With<Moving>)>, Changed<Sprite>),
    >,
) {
    for (sprite, precomputed_hitbox, mut collider, shape_caster) in &mut players {
        let new_collider: avian2d::prelude::Collider = precomputed_hitbox.collider(sprite.flip_x);
        *collider = new_collider;

        if let Some(mut shape_caster) = shape_caster {
            let new_shape = precomputed_hitbox.ground_caster(sprite.flip_x);
            shape_caster.shape = new_shape;
        }
    }
}

