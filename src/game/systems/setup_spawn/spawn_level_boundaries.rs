use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::systems::systems_api::ActiveLevelBounds;
use crate::game::systems::systems_api::LEVEL_BOUNDARY_THICKNESS;

pub(crate) fn spawn_level_boundaries(commands: &mut Commands, level_bounds: ActiveLevelBounds) {
    let half_thickness = LEVEL_BOUNDARY_THICKNESS * 0.5;
    let vertical_center_y = (level_bounds.bottom + level_bounds.top) * 0.5;
    let horizontal_center_x = (level_bounds.left + level_bounds.right) * 0.5;
    let vertical_wall_height =
        (level_bounds.top - level_bounds.bottom) + LEVEL_BOUNDARY_THICKNESS * 2.0;
    let horizontal_wall_width =
        (level_bounds.right - level_bounds.left) + LEVEL_BOUNDARY_THICKNESS * 2.0;

    let walls = [
        (
            "Left",
            Vec3::new(level_bounds.left - half_thickness, vertical_center_y, 50.0),
            Vec2::new(LEVEL_BOUNDARY_THICKNESS, vertical_wall_height),
        ),
        (
            "Right",
            Vec3::new(level_bounds.right + half_thickness, vertical_center_y, 50.0),
            Vec2::new(LEVEL_BOUNDARY_THICKNESS, vertical_wall_height),
        ),
        (
            "Bottom",
            Vec3::new(
                horizontal_center_x,
                level_bounds.bottom - half_thickness,
                50.0,
            ),
            Vec2::new(horizontal_wall_width, LEVEL_BOUNDARY_THICKNESS),
        ),
        (
            "Top",
            Vec3::new(horizontal_center_x, level_bounds.top + half_thickness, 50.0),
            Vec2::new(horizontal_wall_width, LEVEL_BOUNDARY_THICKNESS),
        ),
    ];

    for (name, translation, size) in walls {
        commands.spawn((
            Name::new(format!("LevelBoundary:{name}")),
            Transform::from_translation(translation),
            Collider::rectangle(size.x, size.y),
            RigidBody::Static,
            SpawnedLevelEntity,
        ));
    }
}
