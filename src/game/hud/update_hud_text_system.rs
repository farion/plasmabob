use bevy::prelude::*;

use crate::game::hud::components::{
    ScoreTextShadowUi, ScoreTextUi, TimeTextShadowUi, TimeTextUi,
};
use crate::game::hud::hud_state::HudState;

pub fn update_hud_text_system(
    hud_state: Res<HudState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TimeTextUi>>,
        Query<&mut Text, With<TimeTextShadowUi>>,
        Query<&mut Text, With<ScoreTextUi>>,
        Query<&mut Text, With<ScoreTextShadowUi>>,
    )>,
) {
    let time_text = format_mm_ss(hud_state.level_seconds.max(0.0));
    for mut text in &mut texts.p0() {
        text.0 = time_text.clone();
    }
    for mut text in &mut texts.p1() {
        text.0 = time_text.clone();
    }

    let score_text = format!("Score: {}", hud_state.score);
    for mut text in &mut texts.p2() {
        text.0 = score_text.clone();
    }
    for mut text in &mut texts.p3() {
        text.0 = score_text.clone();
    }
}

fn format_mm_ss(total_seconds: f32) -> String {
    let total = total_seconds.floor() as u64;
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::format_mm_ss;

    #[test]
    fn formats_minutes_and_seconds() {
        assert_eq!(format_mm_ss(0.0), "00:00");
        assert_eq!(format_mm_ss(9.9), "00:09");
        assert_eq!(format_mm_ss(150.0), "02:30");
    }
}


