use bevy::prelude::*;

use crate::app_model::AppState;
use crate::game::systems::gameplay::apply_melee_attack_contact_damage;
use crate::game::systems::gameplay::apply_state_animation;
use crate::game::systems::gameplay::configure_player_controller;
use crate::game::systems::gameplay::control_moving_entities;
use crate::game::systems::gameplay::control_player;
use crate::game::systems::gameplay::count_hostile_deaths;
use crate::game::systems::gameplay::despawn_dead_entities;
use crate::game::systems::gameplay::detect_player_collectibles;
use crate::game::systems::gameplay::detect_player_defeated;
use crate::game::systems::gameplay::detect_player_reached_exit;
use crate::game::systems::gameplay::disable_dead_npc_collisions;
use crate::game::systems::gameplay::maintain_player_fight_state;
use crate::game::systems::gameplay::set_hostile_fight_state_on_player_contact;
use crate::game::systems::gameplay::shoot_plasma;
use crate::game::systems::gameplay::sync_death_state_from_health;
use crate::game::systems::gameplay::sync_player_hitbox_orientation;
use crate::game::systems::gameplay::sync_state_hitboxes;
use crate::game::systems::gameplay::tick_fight_state_timers;
use crate::game::systems::gameplay::tick_hit_state_timers;
use crate::game::systems::gameplay::tick_invincibility_timers;
use crate::game::systems::gameplay::tick_melee_attack_state_timers;
use crate::game::systems::gameplay::update_grounded;
use crate::game::systems::maintenance::cleanup_game_view;
use crate::game::systems::maintenance::toggle_debug_overlay;
use crate::game::systems::maintenance::toggle_hitbox_debug_lines;
use crate::game::systems::maintenance::update_debug_stats_labels;
use crate::game::systems::maintenance::{draw_hitbox_debug_lines, pause_menu};
use crate::game::systems::presentation::apply_parallax_from_camera;
use crate::game::systems::presentation::attach_parallax_anchors;
use crate::game::systems::presentation::follow_player_with_camera;
use crate::game::systems::presentation::health_floating;
use crate::game::systems::presentation::play_hostile_death_quotes;
use crate::game::systems::presentation::snap_camera_to_player;
use crate::game::systems::presentation::tick_level_time;
use crate::game::systems::presentation::types::LevelTimer;
use crate::game::systems::presentation::update_dust_particles;
use crate::game::systems::presentation::update_level_hud;
use crate::game::systems::presentation::update_plasma_beams;
use crate::game::systems::presentation::update_plasma_impact_particles;
use crate::game::systems::presentation::update_player_health_hud;
use crate::game::systems::setup_spawn::setup_game_view;
use crate::game::systems::setup_spawn::spawn_level_hud;
use crate::game::systems::setup_spawn::spawn_player_health_hud;
use crate::game::systems::setup_spawn::spawn_terrain_background_tiles;
use crate::game::systems::systems_api::QuoteCooldown;

pub struct GameViewPlugin;

// ...existing code...

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        // Define ordered SystemSets for gameplay so Scheduling is explicit and extensible
        #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
        pub enum GameViewSet {
            Gameplay,
            Presentation,
            Maintenance,
        }

        app.add_systems(
            OnEnter(AppState::GameView),
            (
                setup_game_view::setup_game_view,
                snap_camera_to_player::snap_camera_to_player,
                configure_player_controller::configure_player_controller,
                spawn_player_health_hud::spawn_player_health_hud,
                spawn_level_hud::spawn_level_hud,
            )
                .chain(),
        )
        .init_resource::<QuoteCooldown>()
        // Floating health change markers are inserted as components by combat
        // systems (RecentHealthChange). No separate event type is required.
        .init_resource::<PauseMenuState>()
        .init_resource::<LevelTimer>()
        .add_systems(
            Update,
            pause_menu::update_pause_menu.run_if(in_state(AppState::GameView)),
        )
        // Configure ordered sets for update-stage: Gameplay -> Presentation -> Maintenance
        .configure_sets(
            Update,
            (GameViewSet::Gameplay, GameViewSet::Presentation, GameViewSet::Maintenance),
        )
        // Gameplay: core simulation and state changes
        .add_systems(
            Update,
            (
                spawn_terrain_background_tiles::spawn_terrain_background_tiles,
                update_grounded::update_grounded,
                (control_player::control_player, sync_player_hitbox_orientation::sync_player_hitbox_orientation)
                    .chain()
                    .before(shoot_plasma::shoot_plasma),
                control_moving_entities::control_moving_entities,
                tick_invincibility_timers::tick_invincibility_timers,
                apply_melee_attack_contact_damage::apply_meele_attack_contact_damage,
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
                    tick_melee_attack_state_timers::tick_melee_attack_state_timers,
                    sync_state_hitboxes::sync_state_hitboxes,
                    apply_state_animation::apply_state_animation,
                    despawn_dead_entities::despawn_dead_entities,
                ),
                tick_level_time::tick_level_time,
                (
                    detect_player_defeated::detect_player_defeated,
                    detect_player_collectibles::detect_player_collectibles,
                    detect_player_reached_exit::detect_player_reached_exit,
                ),
            )
                .in_set(GameViewSet::Gameplay)
                .run_if(gameplay_active)
                .run_if(in_state(AppState::GameView)),
        )
        // Presentation: visuals, HUD and particle updates
        .add_systems(
            Update,
            (
                attach_parallax_anchors::attach_parallax_anchors,
                update_dust_particles::update_dust_particles,
                update_level_hud::update_level_hud,
                update_player_health_hud::update_player_health_hud,
            )
                .in_set(GameViewSet::Presentation)
                .run_if(gameplay_active)
                .run_if(in_state(AppState::GameView)),
        )
        // Maintenance: debug overlays and hitbox drawing
        .add_systems(
            Update,
            (
                toggle_hitbox_debug_lines::toggle_hitbox_debug_lines,
                update_debug_stats_labels::update_debug_stats_labels,
                toggle_debug_overlay::toggle_debug_overlay,
                draw_hitbox_debug_lines::draw_hitbox_debug_lines,
            )
                .in_set(GameViewSet::Maintenance)
                .run_if(gameplay_active)
                .run_if(in_state(AppState::GameView)),
        )
        .add_systems(
            PostUpdate,
            (
                follow_player_with_camera::follow_player_with_camera,
                apply_parallax_from_camera::apply_parallax_from_camera,
            )
                .chain()
                .run_if(in_state(AppState::GameView)),
        )
        // Floating health/damage numbers: spawn on events and animate them
        .add_systems(
            Update,
            (
                health_floating::spawn_on_health_change,
                health_floating::animate_floating_texts,
            )
                .in_set(GameViewSet::Presentation)
                .run_if(in_state(AppState::GameView)),
        )
        .add_systems(OnExit(AppState::GameView), cleanup_game_view::cleanup_game_view);
    }
}

fn gameplay_active(modal_state: Res<PauseMenuState>) -> bool {
    !modal_state.is_open && !modal_state.suppress_enter_until_release
}

#[derive(Resource, Default)]
pub(crate) struct PauseMenuState {
    pub(crate) is_open: bool,
    pub(crate) selection: usize,
    pub(crate) suppress_enter_until_release: bool,
}
