use bevy::prelude::*;

use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::components::{Collider, ColliderShape, Health, Orientation, StateMachine};

use super::draw_hitbox_debug_lines::DebugOwner;

/// Marker for text label entities (kept here for convenience).
#[derive(Component)]
pub struct DebugStatsLabel;

/// Update text labels created by the draw system with the latest entity info.
pub(crate) fn update_debug_stats_labels(
    mut query_labels: Query<(
        &DebugOwner,
        &mut Transform,
        &mut Text2d,
        Option<&TextFont>,
        Option<&mut TextColor>,
    ), With<DebugStatsLabel>>,
    owners: Query<(
        &SpawnedLevelEntity,
        &GlobalTransform,
        &Collider,
        Option<&Health>,
        Option<&StateMachine>,
        Option<&Orientation>,
        Option<&crate::game::tags::PlayerTag>,
        Option<&crate::game::tags::EnemyTag>,
    )>,
) {
    for (dbg_owner, mut transform, mut text, text_font_opt, text_color_opt) in &mut query_labels {
        if let Ok((spawned, owner_tf, collider, health_opt, sm_opt, orientation_opt, player_opt, enemy_opt)) = owners.get(dbg_owner.0) {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!("{} : {}", spawned.entity_type, spawned.id));
            if let Some(h) = health_opt {
                lines.push(format!("HP: {}", h.current));
            }
            if let Some(sm) = sm_opt {
                lines.push(format!("State: {:?}", sm.state));
            }
            if let Some(orientation) = orientation_opt {
                // Derive angle in degrees from the surface_alignment Vec2.
                // Vec2::ZERO yields 0°; any non-zero vector yields the angle
                // it makes with the positive X axis (world right).
                let angle_deg = if orientation.surface_alignment.length_squared() > f32::EPSILON {
                    orientation.surface_alignment.y.atan2(orientation.surface_alignment.x).to_degrees()
                } else {
                    0.0
                };
                lines.push(format!("Orientation: {:?} {:.0}°", orientation.facing, angle_deg));
            }
            let joined = lines.join("\n");
            text.0 = joined;

            // Keep the whole text block above the hitbox by lifting the text
            // origin by half of the estimated text block height.
            let line_count = lines.len().max(1) as f32;
            let font_size = text_font_opt.map(|f| f.font_size).unwrap_or(12.0);
            let line_height = font_size * 1.15;
            let text_half_height = line_count * line_height * 0.5;

            // Keep the label directly above the collider top in world space.
            // Only Rectangle colliders exist at runtime now — extract the
            // half extent directly.
            let half_h = match &collider.shape {
                ColliderShape::Rectangle { half_extents } => half_extents.y,
            };
            let owner_pos = owner_tf.translation();
            transform.translation = Vec3::new(
                owner_pos.x + collider.offset.x,
                owner_pos.y + collider.offset.y + half_h + 8.0 + text_half_height,
                owner_pos.z + 0.2,
            );

            // Update color to match category: player=green, enemy=red, non-gameplay=grey, other=pink
            let color = if player_opt.is_some() {
                Color::srgba(0.0, 1.0, 0.0, 1.0)
            } else if enemy_opt.is_some() {
                Color::srgba(1.0, 0.0, 0.0, 1.0)
            } else if spawned.layer.to_ascii_lowercase() != "gameplay" {
                Color::srgba(0.5, 0.5, 0.5, 1.0)
            } else {
                Color::srgba(1.0, 0.0, 1.0, 1.0)
            };
            if let Some(mut text_color) = text_color_opt {
                text_color.0 = color;
            }

        }
    }
}



