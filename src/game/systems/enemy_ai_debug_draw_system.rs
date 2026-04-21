use bevy::prelude::*;

use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::AutoMovement;

/// Draw debug circles for aggro ranges when enabled.
pub fn enemy_ai_debug_draw_system(
    debug_settings: Res<crate::DebugRenderSettings>,
    mut gizmos: Gizmos,
    enemies: Query<(&Transform, &AutoMovement, Option<&AutoRangeAttack>)>,
) {
    if !debug_settings.show_enemy_ai_debug {
        return;
    }

    let purple = Color::srgb(0.6, 0.0, 0.8);
    let red = Color::srgb(1.0, 0.0, 0.0);
    let green = Color::srgb(0.0, 1.0, 0.0);

    for (transform, auto_movement, maybe_range_attack) in &enemies {
        let center = transform.translation.truncate();
        gizmos.circle_2d(center, auto_movement.aggro_range.max(0.0), purple);
        gizmos.circle_2d(center, auto_movement.deaggro_range.max(0.0), green);
        if let Some(range_attack) = maybe_range_attack {
            gizmos.circle_2d(center, range_attack.aggro_range.max(0.0), red);
        }
    }
}
