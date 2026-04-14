use bevy::prelude::*;

use crate::game::components::GameEntity;
use crate::game::hud::components::{LivesContainerUi, LivesHeartUi};
use crate::game::hud::hud_state::HudState;

const HUD_LIVES_ICON_SIZE: f32 = 34.0;

pub fn update_hud_lives_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hud_state: Res<HudState>,
    containers: Query<(Entity, &Children), With<LivesContainerUi>>,
    hearts: Query<Entity, With<LivesHeartUi>>,
) {
    let Some((container_entity, children)) = containers.iter().next() else {
        return;
    };

    let current_count = children
        .iter()
        .filter(|entity| hearts.get(*entity).is_ok())
        .count() as u8;

    if current_count == hud_state.lives {
        return;
    }

    for child in children.iter() {
        if hearts.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }

    let heart_image = asset_server.load("icons/heart.png");
    commands.entity(container_entity).with_children(|parent| {
        for _ in 0..hud_state.lives {
            parent.spawn((
                Node {
                    width: Val::Px(HUD_LIVES_ICON_SIZE),
                    height: Val::Px(HUD_LIVES_ICON_SIZE),
                    ..default()
                },
                ImageNode::new(heart_image.clone()),
                LivesHeartUi,
                GameEntity,
            ));
        }
    });
}




