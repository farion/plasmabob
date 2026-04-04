use bevy::prelude::*;

use crate::game::view_api::GameViewEntity;
use crate::game::systems::hud_types::{
    PlayerHealthBarFill,
    PlayerHealthPercentText,
    PlayerPlasmaCooldownBarFill,
    PlayerPlasmaCooldownPercentText,
};

const PLAYER_HEALTH_BAR_WIDTH: f32 = 260.0;
const HUD_BAR_HEIGHT: f32 = 24.0;
const HUD_BAR_BORDER_WIDTH: f32 = 2.0;
const HUD_BAR_INNER_WIDTH: f32 = PLAYER_HEALTH_BAR_WIDTH - HUD_BAR_BORDER_WIDTH * 2.0;
const HUD_BAR_INNER_HEIGHT: f32 = HUD_BAR_HEIGHT - HUD_BAR_BORDER_WIDTH * 2.0;
const HUD_TEXT_WIDTH: f32 = 56.0;

pub(crate) fn spawn_player_health_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            GameViewEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    GameViewEntity,
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                        width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                        height: Val::Px(HUD_BAR_HEIGHT),
                        border: UiRect::all(Val::Px(HUD_BAR_BORDER_WIDTH)),
                        padding: UiRect::all(Val::Px(HUD_BAR_BORDER_WIDTH)),
                            ..default()
                        },
                        BorderColor::all(Color::WHITE),
                        BackgroundColor(Color::srgb(0.08, 0.02, 0.02)),
                        GameViewEntity,
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Px(HUD_BAR_INNER_WIDTH),
                                height: Val::Px(HUD_BAR_INNER_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            GameViewEntity,
                        ))
                        .with_children(|inner| {
                            inner.spawn((
                                Node {
                                    width: Val::Px(HUD_BAR_INNER_WIDTH),
                                    height: Val::Px(HUD_BAR_INNER_HEIGHT-1.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-3.0),
                                    top: Val::Px(-3.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.9, 0.08, 0.08)),
                                PlayerHealthBarFill,
                                GameViewEntity,
                            ));
                        });
                    });

                    row.spawn((
                        Node {
                            width: Val::Px(HUD_TEXT_WIDTH),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        GameViewEntity,
                    ))
                    .with_children(|text_parent| {
                        text_parent.spawn((
                            Text::new("100%"),
                            crate::TextFont { font_size: 20.0, ..default() },
                            crate::TextColor(Color::WHITE),
                            PlayerHealthPercentText,
                            GameViewEntity,
                        ));
                    });
                });

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    GameViewEntity,
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                            height: Val::Px(HUD_BAR_HEIGHT),
                            border: UiRect::all(Val::Px(HUD_BAR_BORDER_WIDTH)),
                            padding: UiRect::all(Val::Px(HUD_BAR_BORDER_WIDTH)),
                            ..default()
                        },
                        BorderColor::all(Color::WHITE),
                        BackgroundColor(Color::srgb(0.02, 0.04, 0.08)),
                        GameViewEntity,
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Px(HUD_BAR_INNER_WIDTH),
                                height: Val::Px(HUD_BAR_INNER_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            GameViewEntity,
                        ))
                        .with_children(|inner| {
                            inner.spawn((
                                Node {
                                    width: Val::Px(HUD_BAR_INNER_WIDTH),
                                    height: Val::Px(HUD_BAR_INNER_HEIGHT-1.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-3.0),
                                    top: Val::Px(-3.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.55, 1.0)),
                                PlayerPlasmaCooldownBarFill,
                                GameViewEntity,
                            ));
                        });
                    });

                    row.spawn((
                        Node {
                            width: Val::Px(HUD_TEXT_WIDTH),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        GameViewEntity,
                    ))
                    .with_children(|text_parent| {
                        text_parent.spawn((
                            Text::new("100%"),
                            crate::TextFont { font_size: 20.0, ..default() },
                            crate::TextColor(Color::WHITE),
                            PlayerPlasmaCooldownPercentText,
                            GameViewEntity,
                        ));
                    });
                });
        });
}

