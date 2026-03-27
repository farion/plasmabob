use bevy::prelude::*;
use bevy::audio::AudioSource;

#[path = "systems/camera.rs"]
mod camera;
#[path = "systems/cleanup.rs"]
mod cleanup;
#[path = "systems/combat.rs"]
mod combat;
#[path = "systems/debug.rs"]
mod debug;
#[path = "systems/animation.rs"]
mod animation;
#[path = "systems/npc.rs"]
mod npc;
#[path = "systems/player.rs"]
mod player;
#[path = "systems/setup.rs"]
mod setup;

pub struct GameViewPlugin;

const PLAYER_MOVE_SPEED: f32 = 320.0;
const PLAYER_JUMP_SPEED: f32 = 700.0;
const MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN: f32 = 500.0;
const LEVEL_BOUNDARY_THICKNESS: f32 = 64.0;
const PLAYER_SCREEN_X_ANCHOR: f32 = 0.4;
const PLAYER_INVINCIBILITY_SECONDS: f32 = 1.0;

#[derive(Component)]
struct GameViewEntity;

#[derive(Component)]
struct DebugOverlayRoot;

#[derive(Component)]
struct TerrainBackgroundConfig {
    image: Handle<Image>,
}

#[derive(Component)]
struct TerrainBackgroundReady;

#[derive(Component, Default)]
struct Grounded;

/// Attached to a `Text2d` entity that displays stats above a level entity in debug mode.
#[derive(Component)]
struct DebugStatsLabel {
    target: Entity,
}

#[derive(Resource, Debug, Clone, Copy)]
struct ActiveLevelBounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

#[derive(Resource, Default, Clone)]
struct LevelQuotes {
    clips: Vec<Handle<AudioSource>>,
}

impl ActiveLevelBounds {
    fn from_window_and_level_size(window_size: Vec2, level_size: Vec2) -> Self {
        let left = -(window_size.x * 0.5);
        let bottom = -(window_size.y * 0.5);

        Self {
            left,
            right: left + level_size.x,
            bottom,
            top: bottom + level_size.y,
        }
    }

    fn center_x(self) -> f32 {
        (self.left + self.right) * 0.5
    }
}

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::AppState::GameView),
            (
                setup::setup_game_view,
                camera::snap_camera_to_player,
                player::configure_player_controller,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                setup::spawn_terrain_background_tiles,
                player::update_grounded,
                (player::control_player, player::sync_player_hitbox_orientation)
                    .chain()
                    .before(combat::shoot_plasma),
                npc::control_moving_entities,
                combat::tick_invincibility_timers,
                combat::apply_hostile_contact_damage,
                combat::shoot_plasma.before(combat::update_plasma_beams),
                (
                    combat::update_plasma_beams,
                    combat::update_plasma_impact_particles,
                    combat::maintain_player_fight_state,
                )
                    .chain()
                    .before(animation::tick_hit_state_timers)
                    .before(animation::apply_state_animation),
                animation::sync_death_state_from_health,
                combat::play_hostile_death_quotes,
                animation::tick_hit_state_timers,
                animation::apply_state_animation,
                combat::despawn_dead_entities,
                debug::toggle_hitbox_debug_lines,
                debug::update_debug_stats_labels,
                debug::toggle_debug_overlay,
                debug::draw_hitbox_debug_lines,
                combat::return_to_main_menu,
            )
                .run_if(in_state(crate::AppState::GameView)),
        )
        .add_systems(
            PostUpdate,
            camera::follow_player_with_camera.run_if(in_state(crate::AppState::GameView)),
        )
        .add_systems(OnExit(crate::AppState::GameView), cleanup::cleanup_game_view);
    }
}


