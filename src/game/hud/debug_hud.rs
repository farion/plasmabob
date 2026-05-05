use bevy::prelude::*;
use crate::game::debug_stats::DebugStats;
use bevy::diagnostic::DiagnosticsStore;
use bevy::time::Timer;
use bevy::time::TimerMode;

#[derive(Component)]
pub struct DebugFpsRoot;

#[derive(Component)]
pub struct DebugCountersRoot;

pub fn spawn_debug_hud(mut commands: Commands) {
    // Create a root node (invisible by default) and a single Text child that
    // we update every frame. We keep the Text entity as the one with DebugHudRoot
    // so it's easy to query for updates.
    // Position the debug HUD to the right of the main left-side HUD bars.
    // Duplicate small layout constants from spawn_hud_system so the debug
    // panel doesn't overlap the health/plasma/ego bars.
    const HUD_MARGIN: f32 = 20.0;
    const HUD_BAR_W: f32 = 260.0;
    const HUD_BAR_GAP: f32 = 10.0;
    const HUD_ICON_SIZE: f32 = 28.0;

    let left = HUD_MARGIN + HUD_BAR_W + HUD_BAR_GAP + HUD_ICON_SIZE + 8.0;

    // FPS HUD (top)
    let fps_node = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(HUD_MARGIN),
            ..default()
        })
        .id();
    commands.entity(fps_node).with_children(|p| {
        p.spawn((Text::new(""), DebugFpsRoot));
    });

    // Counters HUD (below FPS)
    let counters_node = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(HUD_MARGIN + 28.0),
            ..default()
        })
        .id();
    commands.entity(counters_node).with_children(|p| {
        p.spawn((Text::new(""), DebugCountersRoot));
    });
}

pub fn update_debug_hud(
    mut queries: ParamSet<(
        Query<&mut Text, With<DebugFpsRoot>>,
        Query<&mut Text, With<DebugCountersRoot>>,
    )>,
    mut stats: ResMut<DebugStats>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut fps_timer: Local<Option<Timer>>,
) {

    // Initialize the per-system timer on first run.
    if fps_timer.is_none() {
        *fps_timer = Some(Timer::from_seconds(1.0, TimerMode::Repeating));
    }
    let timer = fps_timer.as_mut().unwrap();
    timer.tick(time.delta());

    // Update the recorded FPS only once per second for stability.
    if timer.just_finished() {
        let fps = diagnostics
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.value())
            .map(|v| v as f32)
            .unwrap_or(0.0_f32);
        stats.fps = fps;
    }

    // Update FPS text (first query in the ParamSet)
    if let Ok(mut text) = queries.p0().single_mut() {
        if stats.show_fps {
            text.0 = format!("FPS: {fps:.1}", fps = stats.fps);
        } else {
            text.0 = String::new();
        }
    }

    // Update Counters text (second query in the ParamSet)
    if let Ok(mut text) = queries.p1().single_mut() {
        if stats.show_counters {
            text.0 = format!(
                "shape-casts: {}\ncandidates: {}",
                stats.projectile_shape_hits_calls, stats.projectile_shape_hit_candidates
            );
        } else {
            text.0 = String::new();
        }
    }
}
