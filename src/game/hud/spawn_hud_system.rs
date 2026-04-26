use bevy::prelude::*;

use crate::game::components::GameEntity;
use crate::game::hud::components::{
    EgoBarFillUi, HealthBarFillUi, HudRoot, LivesContainerUi, LivesHeartUi, PlasmaBarFillUi,
    ScoreTextShadowUi, ScoreTextUi, TimeTextShadowUi, TimeTextUi,
};
use crate::game::hud::hud_state::HudState;
use crate::helper::active_character::ActiveCharacter;
use crate::helper::asset_io::load_character_asset;

const HUD_MARGIN: f32 = 20.0;
const HUD_BAR_W: f32 = 260.0;
const HUD_BAR_H: f32 = 24.0;
const HUD_BAR_BORDER: f32 = 2.0;
const HUD_ICON_SIZE: f32 = 28.0;
const HUD_BAR_GAP: f32 = 10.0;
const HUD_TEXT_SIZE: f32 = 42.0;
const HUD_TEXT_SHADOW_OFFSET: f32 = 2.0;
const HUD_LIVES_ICON_SIZE: f32 = 34.0;
const HUD_LIVES_GAP: f32 = 8.0;

pub fn spawn_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_character: Res<ActiveCharacter>,
    hud_state: Res<HudState>,
    existing_roots: Query<Entity, With<HudRoot>>,
) {
    if existing_roots.iter().next().is_some() {
        return;
    }

    let hud_root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            HudRoot,
            GameEntity,
        ))
        .id();

    commands.entity(hud_root).with_children(|root| {
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(HUD_MARGIN),
                top: Val::Px(HUD_MARGIN),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(HUD_BAR_GAP),
                ..default()
            },
            GameEntity,
        ))
        .with_children(|bars| {
            // Health bar
            bars.spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                GameEntity,
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: Val::Px(HUD_ICON_SIZE),
                        height: Val::Px(HUD_ICON_SIZE),
                        ..default()
                    },
                    ImageNode::new(load_character_asset::<Image>(
                        &asset_server,
                        "icons/heart.png",
                        *active_character,
                    )),
                    GameEntity,
                ));

                row.spawn((
                    Node {
                        width: Val::Px(HUD_BAR_W),
                        height: Val::Px(HUD_BAR_H),
                        border: UiRect::all(Val::Px(HUD_BAR_BORDER)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                    GameEntity,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Px(HUD_BAR_W - HUD_BAR_BORDER * 2.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.88, 0.1, 0.1)),
                        HealthBarFillUi,
                        GameEntity,
                    ));
                });
            });

            // Plasma cooldown bar
            bars.spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                GameEntity,
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: Val::Px(HUD_ICON_SIZE),
                        height: Val::Px(HUD_ICON_SIZE),
                        ..default()
                    },
                    ImageNode::new(load_character_asset::<Image>(
                        &asset_server,
                        "icons/plasma.png",
                        *active_character,
                    )),
                    GameEntity,
                ));

                row.spawn((
                    Node {
                        width: Val::Px(HUD_BAR_W),
                        height: Val::Px(HUD_BAR_H),
                        border: UiRect::all(Val::Px(HUD_BAR_BORDER)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                    GameEntity,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Px(HUD_BAR_W - HUD_BAR_BORDER * 2.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 1.0)),
                        PlasmaBarFillUi,
                        GameEntity,
                    ));
                });
            });

            // Ego bar (placeholder, currently gameplay value defaults to 0)
            bars.spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                GameEntity,
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: Val::Px(HUD_ICON_SIZE),
                        height: Val::Px(HUD_ICON_SIZE),
                        ..default()
                    },
                    ImageNode::new(load_character_asset::<Image>(
                        &asset_server,
                        "icons/ego.png",
                        *active_character,
                    )),
                    GameEntity,
                ));

                row.spawn((
                    Node {
                        width: Val::Px(HUD_BAR_W),
                        height: Val::Px(HUD_BAR_H),
                        border: UiRect::all(Val::Px(HUD_BAR_BORDER)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                    GameEntity,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Px(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.95, 0.8, 0.15)),
                        EgoBarFillUi,
                        GameEntity,
                    ));
                });
            });
        });

        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(HUD_MARGIN),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-120.0)),
                width: Val::Px(240.0),
                height: Val::Px(HUD_TEXT_SIZE + 8.0),
                ..default()
            },
            GameEntity,
        ))
        .with_children(|time_parent| {
            time_parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(HUD_TEXT_SHADOW_OFFSET),
                    top: Val::Px(HUD_TEXT_SHADOW_OFFSET),
                    ..default()
                },
                Text::new("00:00"),
                TextFont {
                    font_size: HUD_TEXT_SIZE,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                TimeTextShadowUi,
                GameEntity,
            ));

            time_parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Text::new("00:00"),
                TextFont {
                    font_size: HUD_TEXT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimeTextUi,
                GameEntity,
            ));
        });

        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(HUD_MARGIN),
                right: Val::Px(HUD_MARGIN),
                width: Val::Px(300.0),
                height: Val::Px(HUD_TEXT_SIZE + 8.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            GameEntity,
        ))
        .with_children(|score_parent| {
            let score_text = format!("Score: {}", hud_state.score);

            score_parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(HUD_TEXT_SHADOW_OFFSET),
                    top: Val::Px(HUD_TEXT_SHADOW_OFFSET),
                    ..default()
                },
                Text::new(score_text.clone()),
                TextFont {
                    font_size: HUD_TEXT_SIZE,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                ScoreTextShadowUi,
                GameEntity,
            ));

            score_parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Text::new(score_text),
                TextFont {
                    font_size: HUD_TEXT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreTextUi,
                GameEntity,
            ));
        });

        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(HUD_MARGIN),
                bottom: Val::Px(HUD_MARGIN),
                column_gap: Val::Px(HUD_LIVES_GAP),
                ..default()
            },
            LivesContainerUi,
            GameEntity,
        ))
        .with_children(|lives| {
            let heart_image = load_character_asset::<Image>(
                &asset_server,
                "icons/heart.png",
                *active_character,
            );
            for _ in 0..hud_state.lives {
                lives.spawn((
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
    });
}

