use bevy::prelude::*;

use crate::app_model::AppState;

/// System-set labels used to order the two phases of GameView initialisation.
///
/// `LoadLevel` validates that level data is available.
/// `Setup` runs second and spawns the visual scene from that resource.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSetupSet {
    /// Ensure level data exists (or redirect to `LoadView`).
    LoadLevel,
    /// Spawn camera position, background, and level entities.
    Setup,
}

pub struct GameViewPlugin;

// When the Game view is entered we must ensure a `CachedLevelDefinition`
// resource is present. If not, we redirect to `LoadView` which performs
// asynchronous level loading.
impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        // Enforce ordering: all Setup systems run after LoadLevel.
        app.configure_sets(
            OnEnter(AppState::GameView),
            GameSetupSet::Setup.after(GameSetupSet::LoadLevel),
        )
        .add_systems(
            OnEnter(AppState::GameView),
            (
                reset_level_stats,
                ensure_level_is_preloaded,
            )
                .chain()
                .in_set(GameSetupSet::LoadLevel),
        )
        .add_systems(
            OnExit(AppState::GameView),
            (cleanup_cached_level, reset_main_camera, reset_music_to_menu),
        )
        .add_plugins(crate::game::hud::HudPlugin)
        .add_plugins(crate::game::setup::SetupPlugin)
        .add_plugins(crate::game::systems::SystemsPlugin);
    }
}

/// If level data is missing when entering `GameView`, redirect to `LoadView`.
///
/// `LoadView` is the single place that performs asynchronous level loading,
/// then transitions back to `GameView` once assets are ready.
fn ensure_level_is_preloaded(
    level_selection: Res<crate::LevelSelection>,
    existing: Option<Res<crate::game::level::types::CachedLevelDefinition>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // If LoadView already populated the resource, nothing to do.
    if existing.is_some() {
        tracing::debug!("ensure_level_is_preloaded: CachedLevelDefinition already present");
        return;
    }
    tracing::info!(
        level = %level_selection.asset_path(),
        "GameView entered without cached level data; redirecting to LoadView"
    );
    next_state.set(AppState::LoadView);
}

fn reset_level_stats(mut level_stats: ResMut<crate::LevelStats>) {
    *level_stats = crate::LevelStats::default();
}

/// Clean up the cached level resource when leaving the Game view to avoid
/// stale data when returning later.
fn cleanup_cached_level(mut commands: Commands) {
    // Remove the resource if present.
    commands.remove_resource::<crate::game::level::types::CachedLevelDefinition>();
}

/// Reset the main camera transform when leaving the Game view so UI/menu
/// views that expect the camera at the origin render correctly.
fn reset_main_camera(mut cameras: Query<&mut Transform, With<crate::MainCamera>>) {
    for mut tf in cameras.iter_mut() {
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
        tf.translation.z = 0.0;
        tf.rotation = Default::default();
    }
}

/// Restore global music to the menu track when leaving the Game view.
fn reset_music_to_menu(
    mut music_request: ResMut<crate::helper::music::MusicRequest>,
) {
    music_request.0 = Some(vec!["music/start.ogg".to_string()]);
}

