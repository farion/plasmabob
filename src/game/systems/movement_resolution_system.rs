use bevy::prelude::*;

use crate::game::components::{Blocking, Collider, ColliderShape, Gravity, RigidBody};
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
        step_grounding.clear_step_contacts();

        let mut position = transform.translation.truncate();
        let mut delta = rigid_body.velocity * dt;

        if gravity.grounded {
            delta += inherited_support_velocity * dt;
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
        ),
        With<Blocking>,
    >,
    step_grounding: &mut GroundingState,
    dt: f32,
    max_ground_dot: f32,
) -> Vec2 {
    if delta_axis.abs() <= f32::EPSILON {
        return position;
    }

    let mut mover_aabb = aabb_from_rect(position + mover_collider.offset, mover_half_extents);

    for (_blocker_entity, blocker_transform, blocker_collider, blocker_rb, blocker_prev) in blockers {
        let Some(blocker_half_extents) = rectangle_half_extents(blocker_collider) else {
            continue;
        };

        let blocker_center = blocker_transform.translation.truncate() + blocker_collider.offset;
        let blocker_aabb = aabb_from_rect(blocker_center, blocker_half_extents);

        if !mover_aabb.overlaps(&blocker_aabb) {
            continue;
        }

        if axis == Vec2::X {
            if delta_axis > 0.0 {
                let penetration = mover_aabb.max.x - blocker_aabb.min.x;
                position.x -= penetration;
            } else {
                let penetration = blocker_aabb.max.x - mover_aabb.min.x;
                position.x += penetration;
            }
            rigid_body.velocity.x = 0.0;
        } else {
            if delta_axis > 0.0 {
                let penetration = mover_aabb.max.y - blocker_aabb.min.y;
                position.y -= penetration;
            } else {
                let penetration = blocker_aabb.max.y - mover_aabb.min.y;
                position.y += penetration;

                let contact_normal = Vec2::Y;
                if contact_normal.dot(Vec2::Y) >= max_ground_dot {
                    step_grounding.support_normal_sum_y += contact_normal.y;
                    step_grounding.support_velocity = blocker_step_velocity(blocker_transform, blocker_rb, blocker_prev, dt);
                }
            }
            rigid_body.velocity.y = 0.0;
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
        _ => None,
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

