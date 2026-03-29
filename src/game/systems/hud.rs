use bevy::prelude::*;

use crate::game::components::health::Health;
use crate::game::components::player::{PlasmaAttack, Player};

use super::GameViewEntity;

const PLAYER_HEALTH_BAR_WIDTH: f32 = 260.0;
const HUD_BAR_HEIGHT: f32 = 24.0;
const HUD_BAR_BORDER_WIDTH: f32 = 2.0;
const HUD_BAR_INNER_WIDTH: f32 = PLAYER_HEALTH_BAR_WIDTH - HUD_BAR_BORDER_WIDTH * 2.0;
const HUD_BAR_INNER_HEIGHT: f32 = HUD_BAR_HEIGHT - HUD_BAR_BORDER_WIDTH * 2.0;
const HUD_TEXT_WIDTH: f32 = 56.0;

#[derive(Component)]
pub(super) struct PlayerHealthBarFill;

#[derive(Component)]
pub(super) struct PlayerHealthPercentText;

#[derive(Component)] 
pub(super) struct PlayerPlasmaCooldownBarFill;

#[derive(Component)]
pub(super) struct PlayerPlasmaCooldownPercentText;

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
                        BorderColor(Color::WHITE),
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
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
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
                        BorderColor(Color::WHITE),
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
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            PlayerPlasmaCooldownPercentText,
                            GameViewEntity,
                        ));
                    });
                });
        });
}

pub(super) fn update_player_health_hud(
    player_query: Query<(&Health, &PlasmaAttack), With<Player>>,
    // Beide Füllbalken tragen unterschiedliche Marker und dürfen nie dieselbe UI-Entity matchen.
    mut bar_fill_query: Query<
        &mut Node,
        (With<PlayerHealthBarFill>, Without<PlayerPlasmaCooldownBarFill>),
    >,
    mut health_percent_text_query: Query<
        &mut Text,
        (With<PlayerHealthPercentText>, Without<PlayerPlasmaCooldownPercentText>),
    >,
    mut plasma_cooldown_bar_fill_query: Query<
        &mut Node,
        (With<PlayerPlasmaCooldownBarFill>, Without<PlayerHealthBarFill>),
    >,
    mut plasma_percent_text_query: Query<
        &mut Text,
        (With<PlayerPlasmaCooldownPercentText>, Without<PlayerHealthPercentText>),
    >,
) {
    let Some((health, plasma_attack)) = player_query.iter().next() else {
        return;
    };

    let health_fraction = health_fraction(health.current, health.max);
    let health_width = filled_bar_width(health_fraction);
    let health_percentage_text = percentage_text(health_fraction);
    let plasma_cooldown_fraction = cooldown_fraction(plasma_attack);
    let plasma_cooldown_width = filled_bar_width(plasma_cooldown_fraction);
    let plasma_percentage_text = percentage_text(plasma_cooldown_fraction);

    for mut bar_fill in &mut bar_fill_query {
        bar_fill.width = Val::Px(health_width);
    }

    for mut percent_text in &mut health_percent_text_query {
        percent_text.0 = health_percentage_text.clone();
    }

    for mut plasma_cooldown_bar_fill in &mut plasma_cooldown_bar_fill_query {
        plasma_cooldown_bar_fill.width = Val::Px(plasma_cooldown_width);
    }

    for mut percent_text in &mut plasma_percent_text_query {
        percent_text.0 = plasma_percentage_text.clone();
    }
}

fn health_fraction(current: i32, max: i32) -> f32 {
    if max <= 0 {
        return 0.0;
    }

    (current as f32 / max as f32).clamp(0.0, 1.0)
}

fn percentage_text(fraction: f32) -> String {
    format!("{}%", (fraction * 100.0).round() as i32)
}

fn filled_bar_width(fraction: f32) -> f32 {
    HUD_BAR_INNER_WIDTH * fraction.clamp(0.0, 1.0)
}

fn cooldown_fraction(plasma_attack: &PlasmaAttack) -> f32 {
    plasma_attack.cooldown.fraction().clamp(0.0, 1.0)
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
        assert_eq!(percentage_text(1.0), "100%");
        assert_eq!(percentage_text(0.755), "76%");
        assert_eq!(percentage_text(0.0), "0%");
    }

    #[test]
    fn computes_filled_bar_width_inside_white_frame() {
        assert_eq!(filled_bar_width(1.0), 256.0);
        assert_eq!(filled_bar_width(0.5), 128.0);
        assert_eq!(filled_bar_width(-1.0), 0.0);
        assert_eq!(filled_bar_width(2.0), 256.0);
    }

    #[test]
    fn tracks_plasma_cooldown_fraction() {
        let mut plasma_attack = PlasmaAttack::new(400.0, 10);
        assert_eq!(cooldown_fraction(&plasma_attack), 1.0);

        plasma_attack.cooldown.reset();
        assert_eq!(cooldown_fraction(&plasma_attack), 0.0);

        plasma_attack
            .cooldown
            .tick(std::time::Duration::from_secs_f32(0.25));
        assert_eq!(cooldown_fraction(&plasma_attack), 0.5);
    }

    #[test]
    fn update_system_initializes_without_node_query_conflict() {
        let mut app = App::new();
        app.add_systems(Update, update_player_health_hud);

        app.world_mut().spawn((
            Player,
            Health { current: 80, max: 100 },
            PlasmaAttack::new(400.0, 10),
        ));

        app.world_mut().spawn((
            Node {
                width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                ..default()
            },
            PlayerHealthBarFill,
        ));

        app.world_mut().spawn((Text::new("100%"), PlayerHealthPercentText));

        app.world_mut().spawn((
            Node {
                width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                ..default()
            },
            PlayerPlasmaCooldownBarFill,
        ));

        app.world_mut()
            .spawn((Text::new("100%"), PlayerPlasmaCooldownPercentText));

        app.update();
    }

    #[test]
    fn update_system_updates_both_bars_and_percentage_texts() {
        let mut app = App::new();
        app.add_systems(Update, update_player_health_hud);

        let mut plasma_attack = PlasmaAttack::new(400.0, 10);
        plasma_attack.cooldown.reset();
        plasma_attack
            .cooldown
            .tick(std::time::Duration::from_secs_f32(0.25));

        app.world_mut().spawn((
            Player,
            Health { current: 80, max: 100 },
            plasma_attack,
        ));

        let health_fill = app
            .world_mut()
            .spawn((Node::default(), PlayerHealthBarFill))
            .id();
        let health_text = app
            .world_mut()
            .spawn((Text::new("0%"), PlayerHealthPercentText))
            .id();
        let plasma_fill = app
            .world_mut()
            .spawn((Node::default(), PlayerPlasmaCooldownBarFill))
            .id();
        let plasma_text = app
            .world_mut()
            .spawn((Text::new("0%"), PlayerPlasmaCooldownPercentText))
            .id();

        app.update();

        let health_fill_node = app.world().entity(health_fill).get::<Node>().unwrap();
        assert_eq!(health_fill_node.width, Val::Px(204.8));

        let health_text_value = &app.world().entity(health_text).get::<Text>().unwrap().0;
        assert_eq!(health_text_value, "80%");

        let plasma_fill_node = app.world().entity(plasma_fill).get::<Node>().unwrap();
        assert_eq!(plasma_fill_node.width, Val::Px(128.0));

        let plasma_text_value = &app.world().entity(plasma_text).get::<Text>().unwrap().0;
        assert_eq!(plasma_text_value, "50%");
    }
}

