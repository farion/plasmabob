pub mod cleanup_game_view;
// combat.rs removed — combat systems are declared individually below.
pub mod tick_invincibility_timers;
pub mod apply_hostile_contact_damage;
pub mod set_hostile_fight_state_on_player_contact;
pub mod shoot_plasma;
pub mod combat_types;
pub mod update_plasma_beams;
pub mod update_plasma_impact_particles;
pub mod maintain_player_fight_state;
pub mod disable_dead_npc_collisions;
pub mod play_hostile_death_quotes;
pub mod count_hostile_deaths;
pub mod despawn_dead_entities;
pub mod detect_player_defeated;
pub mod detect_player_reached_exit;
pub mod detect_player_collectibles;
pub mod hud_types;
pub mod spawn_player_health_hud;
pub mod spawn_level_hud;
pub mod tick_level_time;
pub mod update_level_hud;
pub mod update_player_health_hud;
pub mod toggle_hitbox_debug_lines;
pub mod update_debug_stats_labels;
pub mod toggle_debug_overlay;
pub mod draw_hitbox_debug_lines;
// animation.rs was intentionally removed; animation-related systems are
// declared directly below so callers reference them as
// `crate::game::systems::tick_hit_state_timers::tick_hit_state_timers`, etc.
pub mod sync_death_state_from_health;
pub mod tick_hit_state_timers;
pub mod tick_fight_state_timers;
pub mod apply_state_animation;
pub mod sync_state_hitboxes;
pub mod snap_camera_to_player;
pub mod follow_player_with_camera;
pub mod control_moving_entities;
pub mod player;
pub mod parallax_types;
pub mod attach_parallax_anchors;
pub mod apply_parallax_from_camera;
pub mod setup_game_view;
pub mod setup_helpers;
pub mod spawn_level_boundaries;
pub mod spawn_terrain_background_tiles;
pub mod spawn_overlay;
pub mod setup;
pub mod pause_menu;
pub mod health_floating;

pub mod common;




