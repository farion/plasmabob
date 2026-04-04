use bevy::prelude::*;

use crate::game::components::player::PlasmaAttack;
use crate::game::components::player::Player;
use crate::game::systems::hud_types::{
    PlayerHealthBarFill,
    PlayerHealthPercentText,
    PlayerPlasmaCooldownBarFill,
    PlayerPlasmaCooldownPercentText,
};

use crate::game::systems::common::hud_helpers::{
    health_fraction,
    percentage_text,
    filled_bar_width,
    cooldown_fraction,
};

pub(crate) fn update_player_health_hud(
    player_query: Query<(&crate::game::components::health::Health, &PlasmaAttack), With<Player>>,
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

