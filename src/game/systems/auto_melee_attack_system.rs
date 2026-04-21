use bevy::prelude::*;

use crate::game::components::auto_melee_attack::AutoMeleeAttack;
use crate::game::components::{
    Collider, ColliderShape, ControlledMovement, Damageable, Health, StateMachine, Team,
};
use crate::game::runtime_components::DamagePopupSettings;
use crate::game::systems::damage_popup_system::spawn_damage_popup;

const NEUTRAL_TEAM: &str = "Neutral";

/// Detects AABB collisions between entities with [`AutoMeleeAttack`] and entities
/// with [`Damageable`] (on a different team).
///
/// Each frame the cooldown timer is advanced. When it fires the system checks for
/// overlap with every valid damageable target:
/// - [`Health`] is reduced by `AutoMeleeAttack::damage`.
/// - The target's `Damageable::damaged_timer` is set so `state_machine_update_system`
///   transitions the target to the `Damaged` state.
/// - `AutoMeleeAttack::just_attacked` is set to `true` so `state_machine_update_system`
///   transitions the attacker to the `MeleeAttacking` state.
///
/// # Query disjointness
/// `targets` uses `Without<AutoMeleeAttack>` so an enemy that has both components
/// (attacker + damageable) never appears in both queries simultaneously, which keeps
/// Bevy's borrow checker happy.
pub fn auto_melee_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut attackers: Query<(
        Entity,
        &Transform,
        &Collider,
        &mut AutoMeleeAttack,
        Option<&Team>,
        Option<&StateMachine>,
    )>,
    // Only entities that do NOT have AutoMeleeAttack can be targets (prevents same-archetype aliasing).
    targets: Query<(Entity, &Transform, &Collider, Option<&Team>), Without<AutoMeleeAttack>>,
    state_machines: Query<&StateMachine>,
    mut health_q: Query<&mut Health>,
    mut damageable_q: Query<&mut Damageable>,
    controlled_q: Query<&ControlledMovement>,
    damage_settings: Res<DamagePopupSettings>,
) {
    let dt = time.delta();

    for (
        attacker_entity,
        attacker_transform,
        attacker_collider,
        mut melee,
        attacker_team,
        attacker_sm,
    ) in &mut attackers
    {
        if !melee.enabled {
            continue;
        }

        // Skip dead or dying attackers - they should not deal damage.
        if let Some(sm) = attacker_sm {
            if sm.is_non_interactive() {
                continue;
            }
        }

        // Advance the cooldown timer every frame.
        melee.cooldown.tick(dt);

        // Only check for targets when the cooldown just fired.
        if !melee.cooldown.just_finished() {
            continue;
        }

        let attacker_team_name = attacker_team
            .map(|t| t.name.as_str())
            .unwrap_or(NEUTRAL_TEAM);

        let Some(attacker_half) = rectangle_half_extents(attacker_collider) else {
            continue;
        };
        let attacker_center = attacker_transform.translation.truncate() + attacker_collider.offset;

        let mut hit_any = false;

        for (target_entity, target_transform, target_collider, target_team) in &targets {
            if target_entity == attacker_entity {
                continue;
            }

            if let Ok(target_sm) = state_machines.get(target_entity) {
                if target_sm.is_non_interactive() {
                    continue;
                }
            }

            // No friendly fire — same team cannot be damaged.
            let target_team_name = target_team.map(|t| t.name.as_str()).unwrap_or(NEUTRAL_TEAM);
            if attacker_team_name == target_team_name {
                continue;
            }

            // Only entities that actually have Damageable are valid targets.
            if !damageable_q.contains(target_entity) {
                continue;
            }

            let Some(target_half) = rectangle_half_extents(target_collider) else {
                continue;
            };
            let target_center = target_transform.translation.truncate() + target_collider.offset;

            if !aabb_overlap(attacker_center, attacker_half, target_center, target_half) {
                continue;
            }

            // --- Hit confirmed ---

            // Reduce target health.
            if let Ok(mut hp) = health_q.get_mut(target_entity) {
                hp.damage(melee.damage);

                // Spawn floating damage numbers above the hit target.
                let pos = target_transform.translation + Vec3::new(0.0, 0.0, 20.0);
                let is_controlled = controlled_q.get(target_entity).is_ok();
                spawn_damage_popup(
                    &mut commands,
                    pos,
                    melee.damage as i32,
                    false,
                    is_controlled,
                    &*damage_settings,
                );
            }

            // Trigger the Damaged state on the target via its timer.
            if let Ok(mut dmg) = damageable_q.get_mut(target_entity) {
                dmg.damaged_timer = dmg.damaged_duration_secs;
            }

            hit_any = true;

            tracing::debug!(
                attacker = ?attacker_entity,
                target   = ?target_entity,
                damage   = melee.damage,
                "AutoMeleeAttack hit"
            );
        }

        if hit_any {
            // Signal to state_machine_update_system to enter MeleeAttacking.
            melee.just_attacked = true;
        }
    }
}

// ─── Geometry helpers ─────────────────────────────────────────────────────────

fn rectangle_half_extents(collider: &Collider) -> Option<Vec2> {
    match &collider.shape {
        ColliderShape::Rectangle { half_extents } => Some(*half_extents),
    }
}

/// Returns `true` when two axis-aligned bounding boxes overlap.
fn aabb_overlap(center_a: Vec2, half_a: Vec2, center_b: Vec2, half_b: Vec2) -> bool {
    let diff = (center_a - center_b).abs();
    diff.x < half_a.x + half_b.x && diff.y < half_a.y + half_b.y
}
