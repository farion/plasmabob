use bevy::prelude::*;

use crate::game::components::SpawnedLevelEntity;
use crate::game::components::hitbox::PolygonHitbox;

pub(crate) fn draw_hitbox_debug_lines(
    debug_settings: Res<crate::DebugRenderSettings>,
    mut gizmos: Gizmos,
    hitboxes: Query<(&GlobalTransform, &PolygonHitbox, Option<&Sprite>), With<SpawnedLevelEntity>>,
) {
    if !debug_settings.show_hitbox_lines {
        return;
    }

    for (transform, polygon_hitbox, sprite) in &hitboxes {
        let effective_points =
            polygon_hitbox.effective_points(sprite.map(|s| s.flip_x).unwrap_or(false));

        if effective_points.len() < 2 {
            continue;
        }

        for edge_start in 0..effective_points.len() {
            let edge_end = (edge_start + 1) % effective_points.len();
            let start = transform.transform_point(effective_points[edge_start].extend(0.0));
            let end = transform.transform_point(effective_points[edge_end].extend(0.0));
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}
