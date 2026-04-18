use bevy::prelude::*;

use crate::game::components::{CollectibleEffect, Collider, ControlledMovement, Health, StateMachine, EntityState};
use crate::game::systems::damage_popup_system::spawn_damage_popup;
use crate::game::runtime_components::DamagePopupSettings;
use crate::game::tags::CollectibleTag;

/// Detect overlaps between controlled entities and collectibles. When a pickup
/// occurs, apply the typed CollectibleEffect (currently `heal`) to the picker,
/// remove the collectible's Collider to avoid double-pickup, and transition the
/// collectible's StateMachine to `Collected` to play the 1s pickup animation.
pub fn collectible_collision_system(
    mut commands: Commands,
    mut collectibles: Query<(Entity, &Transform, &Collider, Option<&mut StateMachine>, Option<&CollectibleEffect>), With<CollectibleTag>>,
    mut pickers: Query<(Entity, &Transform, &Collider, Option<&mut Health>), With<ControlledMovement>>,
    damage_settings: Res<DamagePopupSettings>,
    mut stats: ResMut<crate::LevelStats>,
) {
    // Simple AABB overlap test (no continuous collision).
    for (col_ent, col_tr, col_col, col_sm_opt, col_eff_opt) in &mut collectibles {
        // If already collected, skip. Use an immutable borrow to avoid moving the
        // optional Mut<StateMachine> so we can mutate it later when consuming.
        if let Some(sm_ref) = col_sm_opt.as_ref() {
            if sm_ref.is(EntityState::Collected) {
                continue;
            }
        }

        let Some(col_half) = rectangle_half_extents(col_col) else { continue; };
        let col_center = col_tr.translation.truncate() + col_col.offset;

        for (_picker_ent, picker_tr, picker_col, picker_health_opt) in &mut pickers {
            let Some(p_half) = rectangle_half_extents(picker_col) else { continue; };
            let picker_center = picker_tr.translation.truncate() + picker_col.offset;

            if aabb_overlap(col_center, col_half, picker_center, p_half) {
                // Apply effect
                if let Some(eff) = col_eff_opt {
                    if let Some(mut health) = picker_health_opt {
                        health.heal(eff.heal);

                        // Spawn heal popup on the picker (controlled entity)
                        let pos = picker_tr.translation + Vec3::new(0.0, 0.0, 20.0);
                        spawn_damage_popup(&mut commands, pos, eff.heal as i32, true, true, &*damage_settings);
                    }
                }

                // Prevent further pickups in the same frame by removing the collider.
                commands.entity(col_ent).remove::<Collider>();

                // Transition to Collected state if we have a StateMachine. This
                // moves the Mut<StateMachine> out of the option so we can mutate it.
                if let Some(mut sm) = col_sm_opt {
                    sm.set_state(EntityState::Collected);
                } else {
                    // Fallback: no StateMachine => despawn immediately.
                    commands.entity(col_ent).try_despawn();
                }

                stats.collectibles_collected = stats.collectibles_collected.saturating_add(1);
                stats.recompute_score();

                break; // collectible consumed, move to next collectible
            }
        }
    }
}

fn rectangle_half_extents(collider: &Collider) -> Option<Vec2> {
    match &collider.shape {
        crate::game::components::ColliderShape::Rectangle { half_extents } => Some(*half_extents),
    }
}

fn aabb_overlap(c1: Vec2, h1: Vec2, c2: Vec2, h2: Vec2) -> bool {
    let min1 = c1 - h1;
    let max1 = c1 + h1;
    let min2 = c2 - h2;
    let max2 = c2 + h2;
    !(max1.x < min2.x || min1.x > max2.x || max1.y < min2.y || min1.y > max2.y)
}




