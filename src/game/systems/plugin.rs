use bevy::prelude::*;

use crate::app_model::AppState;

use crate::game::systems::enemy_random_patrol_system::enemy_random_patrol_system;
use crate::game::systems::gravity_integration_system::gravity_integration_system;
use crate::game::systems::grounding_evaluation_system::grounding_evaluation_system;
use crate::game::systems::moving_platform_system::moving_platform_system;
use crate::game::systems::movement_resolution_system::movement_resolution_system;
use crate::game::systems::player_control_system::player_control_system;
use crate::game::systems::projectile_collision_system::projectile_collision_system;
use crate::game::systems::track_previous_transform_system::track_previous_transform_system;

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
        .add_systems(
            Update,
            (
                player_control_system.in_set(GameplaySet::Input),
                enemy_random_patrol_system.in_set(GameplaySet::Ai),
                gravity_integration_system.in_set(GameplaySet::Physics),
                moving_platform_system
                    .in_set(GameplaySet::Physics)
                    .before(movement_resolution_system),
                movement_resolution_system.in_set(GameplaySet::Physics),
                grounding_evaluation_system.in_set(GameplaySet::Grounding),
                projectile_collision_system.in_set(GameplaySet::Projectile),
                track_previous_transform_system.in_set(GameplaySet::Finalize),
            ),
        );
    }
}

