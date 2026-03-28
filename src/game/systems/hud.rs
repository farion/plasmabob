use bevy::prelude::*;

use crate::game::components::health::Health;
use crate::game::components::player::Player;

use super::GameViewEntity;

const PLAYER_HEALTH_BAR_WIDTH: f32 = 260.0;

#[derive(Component)]
pub(super) struct PlayerHealthBarFill;

#[derive(Component)]
pub(super) struct PlayerHealthPercentText;

pub(super) fn spawn_player_health_hud(mut commands: Commands) {
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
                        width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                        height: Val::Px(24.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BackgroundColor(Color::srgb(0.15, 0.0, 0.0)),
                    GameViewEntity,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.9, 0.08, 0.08)),
                        PlayerHealthBarFill,
                        GameViewEntity,
                    ));
                });

            parent.spawn((
                Text::new("100%"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.85)),
                PlayerHealthPercentText,
                GameViewEntity,
            ));
        });
}

pub(super) fn update_player_health_hud(
    player_health_query: Query<&Health, With<Player>>,
    mut bar_fill_query: Query<&mut Node, With<PlayerHealthBarFill>>,
    mut percent_text_query: Query<&mut Text, With<PlayerHealthPercentText>>,
) {
    let Some(health) = player_health_query.iter().next() else {
        return;
    };

    let fraction = health_fraction(health.current, health.max);
    let width = PLAYER_HEALTH_BAR_WIDTH * fraction;
    let percentage_text = health_percentage_text(fraction);

    for mut bar_fill in &mut bar_fill_query {
        bar_fill.width = Val::Px(width);
    }

    for mut percent_text in &mut percent_text_query {
        percent_text.0 = percentage_text.clone();
    }
}

fn health_fraction(current: i32, max: i32) -> f32 {
    if max <= 0 {
        return 0.0;
    }

    (current as f32 / max as f32).clamp(0.0, 1.0)
}

fn health_percentage_text(fraction: f32) -> String {
    format!("{}%", (fraction * 100.0).round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_health_fraction() {
        assert_eq!(health_fraction(50, 100), 0.5);
        assert_eq!(health_fraction(200, 100), 1.0);
        assert_eq!(health_fraction(-10, 100), 0.0);
    }

    #[test]
    fn handles_zero_or_negative_max_health() {
        assert_eq!(health_fraction(10, 0), 0.0);
        assert_eq!(health_fraction(10, -5), 0.0);
    }

    #[test]
    fn formats_percentage_text() {
        assert_eq!(health_percentage_text(1.0), "100%");
        assert_eq!(health_percentage_text(0.755), "76%");
        assert_eq!(health_percentage_text(0.0), "0%");
    }
}

