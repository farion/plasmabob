use bevy::prelude::*;
use crate::game::view_api::GameViewEntity;

pub(crate) fn spawn_overlay(
    commands: &mut Commands,
    status_title: String,
    status_detail: String,
    warnings: &[String],
) {
    let warning_text = if warnings.is_empty() {
        "No component warnings".to_string()
    } else {
        format!("Warnings: {}", warnings.join(" | "))
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Px(8.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            Visibility::Hidden,
            crate::game::view_api::DebugOverlayRoot,
            crate::game::view_api::GameViewEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 38.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                crate::i18n::LocalizedText { key: "game.view_title".to_string() },
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(status_title),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.7, 1.0)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(status_detail),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(warning_text),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.35)),
                GameViewEntity,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                crate::i18n::LocalizedText { key: "game.overlay_hint".to_string() },
                GameViewEntity,
            ));
        });
}

