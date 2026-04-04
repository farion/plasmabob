use bevy::prelude::*;
use avian2d::prelude::{ShapeCaster, ShapeHits};

use crate::game::components::player::Player;
use crate::game::view_api::Grounded;
use crate::game::systems::player_helpers::{ensure_dust_particle_image, spawn_dust_burst, dust_origin};

pub(crate) fn update_grounded(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut dust_particle_image: Local<Option<Handle<Image>>>,
    players: Query<(Entity, Has<Grounded>, &ShapeHits, &Transform, &Sprite), (With<Player>, With<ShapeCaster>)>,
) {
    let particle_image = ensure_dust_particle_image(&mut dust_particle_image, &mut images);

    for (player, was_grounded, hits, transform, sprite) in &players {
        let is_grounded = !hits.is_empty();

        if is_grounded && !was_grounded {
            spawn_dust_burst(
                &mut commands,
                dust_origin(transform, sprite),
                &particle_image,
                12,
                player.index_u32() + 100,
                220.0,
            );
        }

        if is_grounded {
            commands.entity(player).insert(Grounded);
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
}

