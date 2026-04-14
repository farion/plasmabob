use bevy::prelude::*;

use crate::app_model::AppState;

use crate::game::systems::animation_system::animation_tick_system;
use crate::game::systems::auto_melee_attack_system::auto_melee_attack_system;
use crate::game::systems::enemy_random_patrol_system::enemy_random_patrol_system;
use crate::game::systems::beam_update_system::beam_update_system;
use crate::game::systems::gravity_integration_system::gravity_integration_system;
use crate::game::systems::grounding_evaluation_system::grounding_evaluation_system;
use crate::game::systems::moving_platform_system::moving_platform_system;
use crate::game::systems::movement_resolution_system::movement_resolution_system;
use crate::game::systems::orientation_update_system::orientation_update_system;
use crate::game::systems::player_control_system::player_control_system;
use crate::game::systems::player_shoot_system::player_shoot_system;
use crate::game::systems::projectile_collision_system::projectile_collision_system;
use crate::game::systems::projectile_movement_system::projectile_movement_system;
use crate::game::systems::sound_system::sound_system;
use crate::game::systems::state_machine_update_system::state_machine_update_system;
use crate::game::systems::track_previous_transform_system::track_previous_transform_system;
use crate::game::systems::toggle_parallax_system::toggle_parallax_system;
use crate::game::systems::maintenance::{
    toggle_hitbox_debug_lines::toggle_hitbox_debug_lines,
    draw_hitbox_debug_lines::draw_hitbox_debug_lines,
    update_debug_stats_labels::update_debug_stats_labels,
};
use crate::game::systems::level_end::check_level_end;
use crate::game::hud::pause_menu::{update_pause_menu, PauseMenuState};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Ai,
    Physics,
    Grounding,
    Projectile,
    Finalize,
}

pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        // Ensure pause menu state resource exists when game systems are added
        app.insert_resource(PauseMenuState::default());

        app.configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Ai,
                GameplaySet::Physics,
                GameplaySet::Grounding,
                GameplaySet::Projectile,
                GameplaySet::Finalize,
            )
                .chain()
                .run_if(in_state(AppState::GameView)),
        )
        // Split into two add_systems calls to stay within Bevy's 20-item tuple limit.
        .add_systems(
            Update,
            (
                player_control_system.in_set(GameplaySet::Input),
                player_shoot_system
                    .in_set(GameplaySet::Input)
                    .after(player_control_system),
                enemy_random_patrol_system.in_set(GameplaySet::Ai),
                auto_melee_attack_system.in_set(GameplaySet::Ai),
                gravity_integration_system.in_set(GameplaySet::Physics),
                moving_platform_system
                    .in_set(GameplaySet::Physics)
                    .before(movement_resolution_system),
                movement_resolution_system.in_set(GameplaySet::Physics),
                grounding_evaluation_system.in_set(GameplaySet::Grounding),
                projectile_collision_system.in_set(GameplaySet::Projectile),
                projectile_movement_system
                    .in_set(GameplaySet::Projectile)
                    .after(projectile_collision_system),
                beam_update_system
                    .in_set(GameplaySet::Projectile)
                    .after(projectile_movement_system),
                track_previous_transform_system.in_set(GameplaySet::Finalize),
            ),
        )
        .add_systems(
            Update,
            (
                orientation_update_system.in_set(GameplaySet::Finalize),
                state_machine_update_system
                    .in_set(GameplaySet::Finalize)
                    .after(orientation_update_system),
                // Animate sprites based on AnimationConfig frame timer.
                animation_tick_system
                    .in_set(GameplaySet::Finalize)
                    .after(state_machine_update_system),
                // Drive per-entity state sounds (start → loop → end sequencing).
                sound_system
                    .in_set(GameplaySet::Finalize)
                    .after(state_machine_update_system),
                // Debug maintenance systems
                toggle_hitbox_debug_lines.in_set(GameplaySet::Input),
                toggle_parallax_system.in_set(GameplaySet::Input),
                update_pause_menu.in_set(GameplaySet::Input),
                // Level end detection should run in the finalize stage so
                // movement/collision systems have already updated entity transforms.
                check_level_end.in_set(GameplaySet::Finalize),
                // Check player death after finalize as well.
                crate::game::systems::level_end::check_player_death.in_set(GameplaySet::Finalize),
                draw_hitbox_debug_lines.in_set(GameplaySet::Finalize),
                update_debug_stats_labels
                    .in_set(GameplaySet::Finalize)
                    .after(draw_hitbox_debug_lines),
            ),
        );
    }
}
