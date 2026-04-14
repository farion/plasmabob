use bevy::prelude::*;

use crate::game::hud::components::{EgoBarFillUi, HealthBarFillUi, PlasmaBarFillUi};
use crate::game::hud::hud_state::HudState;

const HUD_BAR_INNER_W: f32 = 256.0;

pub fn update_hud_bars_system(
    hud_state: Res<HudState>,
    mut fills: ParamSet<(
        Query<&mut Node, With<HealthBarFillUi>>,
        Query<&mut Node, With<PlasmaBarFillUi>>,
        Query<&mut Node, With<EgoBarFillUi>>,
    )>,
) {
    for mut node in &mut fills.p0() {
        node.width = Val::Px(HUD_BAR_INNER_W * hud_state.health_frac.clamp(0.0, 1.0));
    }

    for mut node in &mut fills.p1() {
        node.width = Val::Px(HUD_BAR_INNER_W * hud_state.plasma_cooldown_frac.clamp(0.0, 1.0));
    }

    for mut node in &mut fills.p2() {
        node.width = Val::Px(HUD_BAR_INNER_W * hud_state.ego_frac.clamp(0.0, 1.0));
    }
}

