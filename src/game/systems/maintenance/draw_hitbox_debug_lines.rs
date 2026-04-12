use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::components::{Collider, ColliderShape};
use crate::game::components::StateMachine;
use crate::game::components::Health;
use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::tags::{PlayerTag, EnemyTag, EnvironmentTag, DoodadTag};

/// Marker attached to spawned debug hitbox sprites so they can be cleaned up.
#[derive(Component)]
pub struct DebugHitbox;

/// Back-reference from the debug sprite/text entity to its owning game entity.
#[derive(Component)]
pub struct DebugOwner(pub Entity);

/// System that ensures a debug sprite + label exists for each spawned level
/// entity that has a `Collider`. It also updates transforms / sizes each
/// frame while `DebugRenderSettings.show_hitbox_lines` is true.
pub(crate) fn draw_hitbox_debug_lines(
    mut commands: Commands,
    debug_settings: Res<crate::DebugRenderSettings>,
    mut q_owners: Query<(Entity, &GlobalTransform, &Collider, Option<&PlayerTag>, Option<&EnemyTag>, Option<&EnvironmentTag>, Option<&DoodadTag>, Option<&Health>, Option<&StateMachine>, &SpawnedLevelEntity)>,
    mut q_existing: Query<(Entity, &DebugOwner, &mut Transform, Option<&mut Sprite>), With<DebugHitbox>>,
) {
    if !debug_settings.show_hitbox_lines {
        return;
    }

    // Build a set of owners that already have debug sprites so we don't
    // duplicate them when a new toggle happens.
    let mut existing_owners: HashSet<Entity> = HashSet::new();
    for (_ent, owner, _transform, _sprite) in q_existing.iter() {
        existing_owners.insert(owner.0);
    }

    // Spawn missing debug entities and update existing ones.
    for (owner_ent, owner_tf, collider, is_player, is_enemy, _is_env, _is_doodad, _health, _sm, spawned) in &mut q_owners {
        // Determine color by category
        let color = if is_player.is_some() {
            Color::srgba(0.0, 1.0, 0.0, 0.25) // green
        } else if is_enemy.is_some() {
            Color::srgba(1.0, 0.0, 0.0, 0.25) // red
        } else if spawned.layer.to_ascii_lowercase() != "gameplay" {
            Color::srgba(0.5, 0.5, 0.5, 0.25) // grey
        } else {
            Color::srgba(1.0, 0.0, 1.0, 0.25) // pink
        };

        // Calculate size and position in world space.
        let (half_w, half_h) = match &collider.shape {
            ColliderShape::Rectangle { half_extents } => (half_extents.x, half_extents.y),
            ColliderShape::Circle { radius } => (*radius, *radius),
            ColliderShape::Polygon { points } => {
                if points.is_empty() {
                    (4.0, 4.0)
                } else {
                    let min_x = points.iter().map(|p| p.x).fold(f32::MAX, f32::min);
                    let max_x = points.iter().map(|p| p.x).fold(f32::MIN, f32::max);
                    let min_y = points.iter().map(|p| p.y).fold(f32::MAX, f32::min);
                    let max_y = points.iter().map(|p| p.y).fold(f32::MIN, f32::max);
                    ((max_x - min_x) * 0.5, (max_y - min_y) * 0.5)
                }
            }
        };
        let size = Vec2::new(half_w * 2.0, half_h * 2.0);
        let owner_pos = owner_tf.translation();
        let dbg_pos = Vec3::new(owner_pos.x + collider.offset.x, owner_pos.y + collider.offset.y, owner_pos.z + 0.1);

        // If a debug sprite already exists for this owner, update it. Otherwise spawn.
        if existing_owners.contains(&owner_ent) {
            // Update path: find the matching debug entity and update its transform / sprite color / size
            for (_dbg_ent, dbg_owner, mut transform, sprite_opt) in q_existing.iter_mut() {
                if dbg_owner.0 == owner_ent {
                    transform.translation = dbg_pos;
                    if let Some(mut sprite) = sprite_opt {
                        sprite.color = color;
                        sprite.custom_size = Some(size);
                    }
                }
            }
        } else {
            // Spawn a simple transparent sprite as hitbox visual. We reuse a
            // plain ColorMaterial and a unit quad scaled via Transform.scale.
            let sprite = Sprite {
                color,
                custom_size: Some(size),
                ..Default::default()
            };

            // Spawn the debug sprite entity
            commands.spawn((
                sprite,
                Transform::from_translation(dbg_pos),
                DebugHitbox,
                DebugOwner(owner_ent),
            ));

            // Also spawn a world-space text label above the hitbox.
            let label_pos = Vec3::new(dbg_pos.x, dbg_pos.y + half_h + 8.0, dbg_pos.z + 0.1);
            commands.spawn((
                Text2d::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_translation(label_pos),
                DebugOwner(owner_ent),
                crate::game::systems::maintenance::update_debug_stats_labels::DebugStatsLabel,
            ));
        }
    }
}


