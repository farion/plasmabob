use bevy::prelude::*;

use crate::game::systems::systems_api::GameViewEntity;
use crate::game::systems::presentation::types::{LevelKillsText, LevelTimeText};

pub(crate) fn spawn_level_hud(mut commands: Commands) {
    // Top-center: level time
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(8.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GameViewEntity,
    ))
    .with_children(|parent| {
                parent.spawn((
                    Text::new("0:00"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::WHITE),
                    LevelTimeText,
                    GameViewEntity,
                ));
    });

    // Top-right: kills X/total
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(8.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..default()
        },
        GameViewEntity,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("0/0"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            LevelKillsText,
            GameViewEntity,
        ));
    });
}

