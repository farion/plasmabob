use bevy::prelude::*;

use crate::game::components::{Blocking, Collider, ColliderShape, Gravity, RigidBody, StateMachine};
use crate::game::runtime_components::{GroundingState, PreviousTransform};

const MAX_GROUND_ANGLE_DEG: f32 = 45.0;

pub fn movement_resolution_system(
    mut commands: Commands,
    time: Res<Time>,
    mut movers: Query<
        (
            Entity,
            &mut Transform,
            &Collider,
            &mut RigidBody,
            &Gravity,
            Option<&mut GroundingState>,
        ),
        Without<Blocking>,
    >,
    blockers: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&RigidBody>,
            Option<&PreviousTransform>,
            Option<&StateMachine>,
        ),
        With<Blocking>,
    >,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let max_ground_dot = MAX_GROUND_ANGLE_DEG.to_radians().cos();

    for (entity, mut transform, collider, mut rigid_body, gravity, grounding_state) in &mut movers {
        let Some(mover_half_extents) = rectangle_half_extents(collider) else {
            continue;
        };

        let mut step_grounding = grounding_state.as_deref().copied().unwrap_or_default();
        let inherited_support_velocity = step_grounding.support_velocity;
        let inherited_support_entity = step_grounding.support_entity;
        step_grounding.clear_step_contacts();

        let mut position = transform.translation.truncate();
        let mut delta = rigid_body.velocity * dt;

        if gravity.grounded {
            // Use the *current* velocity of the same support entity when
            // available. This prevents one stale frame of opposite carry at
            // platform direction changes.
            let carry_velocity = if let Some(support_entity) = inherited_support_entity {
                blockers
                    .get(support_entity)
                    .ok()
                    .map(|(_entity, transform, _collider, rb, prev, _sm)| {
                        blocker_step_velocity(transform, rb, prev, dt)
                    })
                    .unwrap_or(inherited_support_velocity)
            } else {
                inherited_support_velocity
            };
            delta += carry_velocity * dt;
        }

        position.x += delta.x;
        position = resolve_axis(
            position,
            Vec2::X,
            delta.x,
            collider,
            mover_half_extents,
            &mut rigid_body,
            &blockers,
            &mut step_grounding,
            dt,
            max_ground_dot,
        );

        position.y += delta.y;
        position = resolve_axis(
            position,
            Vec2::Y,
            delta.y,
            collider,
            mover_half_extents,
            &mut rigid_body,
            &blockers,
            &mut step_grounding,
            dt,
            max_ground_dot,
        );

        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if let Some(mut grounding_state) = grounding_state {
            *grounding_state = step_grounding;
        } else {
            commands.entity(entity).insert(step_grounding);
        }
    }
}

fn resolve_axis(
    mut position: Vec2,
    axis: Vec2,
    delta_axis: f32,
    mover_collider: &Collider,
    mover_half_extents: Vec2,
    rigid_body: &mut RigidBody,
    blockers: &Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&RigidBody>,
            Option<&PreviousTransform>,
            Option<&StateMachine>,
        ),
        With<Blocking>,
    >,
    step_grounding: &mut GroundingState,
    dt: f32,
    max_ground_dot: f32,
) -> Vec2 {
    // For X: skip entirely when there is no horizontal movement (optimisation).
    // For Y: always run so that an upward-moving platform can push the mover up
    // even when the mover's own Y-delta is near zero.
    if axis == Vec2::X && delta_axis.abs() <= f32::EPSILON {
        return position;
    }

    let mut mover_aabb = aabb_from_rect(position + mover_collider.offset, mover_half_extents);

    for (blocker_entity, blocker_transform, blocker_collider, blocker_rb, blocker_prev, blocker_sm) in blockers {
        if blocker_sm.is_some_and(|sm| sm.is_non_interactive()) {
            continue;
        }

        let Some(blocker_half_extents) = rectangle_half_extents(blocker_collider) else {
            continue;
        };

        let blocker_center = blocker_transform.translation.truncate() + blocker_collider.offset;
        let blocker_aabb = aabb_from_rect(blocker_center, blocker_half_extents);

        if !mover_aabb.overlaps(&blocker_aabb) {
            continue;
        }

        if axis == Vec2::X {
            // Compute overlap extents along both axes (positive if overlapping).
            let overlap_x = mover_aabb.max.x.min(blocker_aabb.max.x) - mover_aabb.min.x.max(blocker_aabb.min.x);
            let overlap_y = mover_aabb.max.y.min(blocker_aabb.max.y) - mover_aabb.min.y.max(blocker_aabb.min.y);

            // Centres to decide which object is above the other.
            let mover_center = (mover_aabb.min + mover_aabb.max) * 0.5;
            let blocker_center = (blocker_aabb.min + blocker_aabb.max) * 0.5;

            // small epsilon to avoid floating-point edge cases
            let eps = 1e-3;

            // If the mover is above the blocker and the vertical overlap is
            // small compared to horizontal overlap, treat this as ground-contact
            // and allow horizontal motion (do not apply X correction). This
            // enables walking along the top of small/rounded objects like barrels.
            if mover_center.y > blocker_center.y && overlap_y + eps < overlap_x {
                // ground-like contact: do not block horizontal motion
            } else {
                // Side contact: resolve horizontally, but do not correct more
                // than the movement delta to avoid teleporting through objects.
                let penetration_x = if delta_axis > 0.0 {
                    mover_aabb.max.x - blocker_aabb.min.x
                } else {
                    blocker_aabb.max.x - mover_aabb.min.x
                };
                if delta_axis > 0.0 {
                    let correction = penetration_x.min(delta_axis.abs());
                    position.x -= correction;
                } else {
                    let correction = penetration_x.min(delta_axis.abs());
                    position.x += correction;
                }
                rigid_body.velocity.x = 0.0;
            }
        } else {
            // Y axis: use the spatial relationship between mover and blocker
            // centres to determine whether this is a floor or a ceiling contact.
            // Using delta_axis direction was wrong when a platform pushes the
            // mover upward (positive delta) but is spatially *below* the mover.
            let mover_center_y = (mover_aabb.min.y + mover_aabb.max.y) * 0.5;
            let blocker_center_y = (blocker_aabb.min.y + blocker_aabb.max.y) * 0.5;

            if mover_center_y >= blocker_center_y {
                // Floor contact: mover is on top of the blocker.
                // Apply the FULL penetration without capping to delta_axis so
                // that a fast-moving upward platform can push the mover up even
                // when the mover's own Y-delta is much smaller than the platform
                // displacement.
                let penetration = blocker_aabb.max.y - mover_aabb.min.y;
                if penetration > 0.0 {
                    position.y += penetration;

                    let contact_normal = Vec2::Y;
                    if contact_normal.dot(Vec2::Y) >= max_ground_dot {
                        step_grounding.support_normal_sum_y += contact_normal.y;
                        step_grounding.support_velocity = blocker_step_velocity(blocker_transform, blocker_rb, blocker_prev, dt);
                        step_grounding.support_entity = Some(blocker_entity);
                    }

                    // Only cancel downward velocity; preserve upward velocity so
                    // the mover is not slowed down by the platform and can still
                    // jump normally.
                    if rigid_body.velocity.y < 0.0 {
                        rigid_body.velocity.y = 0.0;
                    }
                }
            } else {
                // Ceiling contact: mover is below the blocker.
                // Cap the correction to the movement delta to avoid teleporting
                // the mover through a static ceiling it was already touching.
                let penetration = mover_aabb.max.y - blocker_aabb.min.y;
                if penetration > 0.0 {
                    let correction = if delta_axis.abs() > f32::EPSILON {
                        penetration.min(delta_axis.abs())
                    } else {
                        penetration
                    };
                    position.y -= correction;

                    // Only cancel upward velocity on ceiling contact.
                    if rigid_body.velocity.y > 0.0 {
                        rigid_body.velocity.y = 0.0;
                    }
                }
            }
        }

        mover_aabb = aabb_from_rect(position + mover_collider.offset, mover_half_extents);
    }

    position
}

fn blocker_step_velocity(
    blocker_transform: &Transform,
    blocker_rb: Option<&RigidBody>,
    blocker_prev: Option<&PreviousTransform>,
    dt: f32,
) -> Vec2 {
    if let Some(previous) = blocker_prev {
        let current = blocker_transform.translation.truncate();
        return (current - previous.position) / dt.max(f32::EPSILON);
    }

    blocker_rb.map(|rb| rb.velocity).unwrap_or(Vec2::ZERO)
}

fn rectangle_half_extents(collider: &Collider) -> Option<Vec2> {
    match &collider.shape {
        ColliderShape::Rectangle { half_extents } => Some(*half_extents),
    }
}

#[derive(Debug, Clone, Copy)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

fn aabb_from_rect(center: Vec2, half_extents: Vec2) -> Aabb {
    Aabb {
        min: center - half_extents,
        max: center + half_extents,
    }
}

