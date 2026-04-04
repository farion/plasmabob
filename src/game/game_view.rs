use bevy::prelude::*;

use crate::game::systems::snap_camera_to_player;
use crate::game::systems::follow_player_with_camera;
use crate::game::systems::cleanup_game_view;
use crate::game::systems::tick_invincibility_timers;
use crate::game::systems::apply_hostile_contact_damage;
use crate::game::systems::set_hostile_fight_state_on_player_contact;
use crate::game::systems::shoot_plasma;
use crate::game::systems::update_plasma_beams;
use crate::game::systems::update_plasma_impact_particles;
use crate::game::systems::maintain_player_fight_state;
use crate::game::systems::disable_dead_npc_collisions;
use crate::game::systems::play_hostile_death_quotes;
use crate::game::systems::count_hostile_deaths;
use crate::game::systems::despawn_dead_entities;
use crate::game::systems::detect_player_defeated;
use crate::game::systems::detect_player_collectibles;
use crate::game::systems::detect_player_reached_exit;
use crate::game::systems::hud;
use crate::game::systems::debug;
use crate::game::systems::npc;
use crate::game::systems::player;
use crate::game::systems::sync_death_state_from_health;
use crate::game::systems::tick_hit_state_timers;
use crate::game::systems::tick_fight_state_timers;
use crate::game::systems::apply_state_animation;
use crate::game::systems::sync_state_hitboxes;
use crate::game::systems::parallax;
use crate::game::systems::setup;
use crate::game::systems::pause_menu;
use crate::game::systems::health_floating;

use crate::game::view_api::{PauseMenuState, QuoteCooldown};

pub struct GameViewPlugin;

// ...existing code...


impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::AppState::GameView),
            (
                setup::setup_game_view,
                snap_camera_to_player::snap_camera_to_player,
                player::configure_player_controller,
                hud::spawn_player_health_hud,
                hud::spawn_level_hud,
            )
                .chain(),
        )
        .init_resource::<QuoteCooldown>()
        // Floating health change markers are inserted as components by combat
        // systems (RecentHealthChange). No separate event type is required.
        .init_resource::<PauseMenuState>()
        .init_resource::<hud::LevelTimer>()
        .add_systems(
            Update,
            pause_menu::update_pause_menu.run_if(in_state(crate::AppState::GameView)),
        )
        .add_systems(
            Update,
            (
                setup::spawn_terrain_background_tiles,
                parallax::attach_parallax_anchors,
                player::update_grounded,
                player::update_dust_particles,
                (player::control_player, player::sync_player_hitbox_orientation)
                    .chain()
                    .before(shoot_plasma::shoot_plasma),
                npc::control_moving_entities,
                tick_invincibility_timers::tick_invincibility_timers,
                apply_hostile_contact_damage::apply_hostile_contact_damage,
                set_hostile_fight_state_on_player_contact::set_hostile_fight_state_on_player_contact,
                (
                    shoot_plasma::shoot_plasma,
                    update_plasma_beams::update_plasma_beams,
                    update_plasma_impact_particles::update_plasma_impact_particles,
                    maintain_player_fight_state::maintain_player_fight_state,
                )
                    .chain()
                    .before(tick_hit_state_timers::tick_hit_state_timers)
                    .before(apply_state_animation::apply_state_animation),
                (
                    sync_death_state_from_health::sync_death_state_from_health,
                    disable_dead_npc_collisions::disable_dead_npc_collisions,
                    play_hostile_death_quotes::play_hostile_death_quotes,
                    // Count deaths for level stats before we despawn the entities.
                    count_hostile_deaths::count_hostile_deaths,
                    tick_hit_state_timers::tick_hit_state_timers,
                    tick_fight_state_timers::tick_fight_state_timers,
                    sync_state_hitboxes::sync_state_hitboxes,
                    apply_state_animation::apply_state_animation,
                    despawn_dead_entities::despawn_dead_entities,
                ),
                debug::toggle_hitbox_debug_lines,
                debug::update_debug_stats_labels,
                debug::toggle_debug_overlay,
                debug::draw_hitbox_debug_lines,
                hud::tick_level_time,
                hud::update_level_hud,
                hud::update_player_health_hud,
                (
                    detect_player_defeated::detect_player_defeated,
                    detect_player_collectibles::detect_player_collectibles,
                    detect_player_reached_exit::detect_player_reached_exit,
                ),
            )
                .run_if(gameplay_active)
                .run_if(in_state(crate::AppState::GameView)),
        )
        .add_systems(
            PostUpdate,
            (
                follow_player_with_camera::follow_player_with_camera,
                parallax::apply_parallax_from_camera,
            )
                .chain()
                .run_if(in_state(crate::AppState::GameView)),
        )
        // Floating health/damage numbers: spawn on events and animate them
        .add_systems(
            Update,
            (
                health_floating::spawn_on_health_change,
                health_floating::animate_floating_texts,
            )
                .run_if(in_state(crate::AppState::GameView)),
        )
        .add_systems(OnExit(crate::AppState::GameView), cleanup_game_view::cleanup_game_view);
    }
}

fn gameplay_active(modal_state: Res<PauseMenuState>) -> bool {
    !modal_state.is_open && !modal_state.suppress_enter_until_release
}


