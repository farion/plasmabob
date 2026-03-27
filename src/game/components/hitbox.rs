use bevy::prelude::*;
use avian2d::prelude::Collider;

use crate::game::level::EntityTypeDefinition;

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct PolygonHitbox {
    pub(crate) points: Vec<Vec2>,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct PrecomputedPlayerHitbox {
    normal_collider: Collider,
    flipped_collider: Collider,
    normal_ground_caster: Collider,
    flipped_ground_caster: Collider,
}

pub(crate) const PLAYER_GROUND_CASTER_SCALE: f32 = 0.99;

impl PrecomputedPlayerHitbox {
    pub(crate) fn from_polygon_hitbox(hitbox: &PolygonHitbox) -> Self {
        let normal_points = hitbox.points.clone();
        let flipped_points = hitbox.mirrored_points();

        let normal_collider = collider_from_points(normal_points);
        let flipped_collider = collider_from_points(flipped_points);

        let mut normal_ground_caster = normal_collider.clone();
        normal_ground_caster.set_scale(Vec2::splat(PLAYER_GROUND_CASTER_SCALE), 10);

        let mut flipped_ground_caster = flipped_collider.clone();
        flipped_ground_caster.set_scale(Vec2::splat(PLAYER_GROUND_CASTER_SCALE), 10);

        Self {
            normal_collider,
            flipped_collider,
            normal_ground_caster,
            flipped_ground_caster,
        }
    }

    pub(crate) fn collider(&self, flip_x: bool) -> Collider {
        if flip_x {
            self.flipped_collider.clone()
        } else {
            self.normal_collider.clone()
        }
    }

    pub(crate) fn ground_caster(&self, flip_x: bool) -> Collider {
        if flip_x {
            self.flipped_ground_caster.clone()
        } else {
            self.normal_ground_caster.clone()
        }
    }
}

impl PolygonHitbox {
    pub(crate) fn effective_points(&self, flip_x: bool) -> Vec<Vec2> {
        if flip_x {
            self.mirrored_points()
        } else {
            self.points.clone()
        }
    }

    pub(crate) fn mirrored_points(&self) -> Vec<Vec2> {
        self.points.iter().rev().map(|point| Vec2::new(-point.x, point.y)).collect()
    }

}

pub(crate) fn from_entity_type(entity_type: &EntityTypeDefinition) -> Result<PolygonHitbox, String> {
    Ok(PolygonHitbox {
        points: entity_type.centered_hitbox_polygon()?,
    })
}

pub(crate) fn collider_from_points(points: Vec<Vec2>) -> Collider {
    let indices: Vec<[u32; 2]> = (0..points.len())
        .map(|i| [i as u32, ((i + 1) % points.len()) as u32])
        .collect();

    Collider::convex_decomposition(points, indices)
}


