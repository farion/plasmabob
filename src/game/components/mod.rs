use std::collections::HashMap;

use avian2d::prelude::{CollidingEntities, LinearVelocity, LockedAxes, RigidBody, CollisionLayers, LayerMask};
use bevy::prelude::*;

use crate::game::level::{EntityDefinition, EntityTypeDefinition};

pub(crate) mod collision;
pub(crate) mod doodad;
pub(crate) mod exit;
pub(crate) mod floor;
pub(crate) mod health;
pub(crate) mod hostile;
pub(crate) mod hitbox;
pub(crate) mod animation;
pub(crate) mod moving;
pub(crate) mod npc;
pub(crate) mod plasma;
pub(crate) mod player;

#[derive(Component)]
pub(crate) struct SpawnedLevelEntity;

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct LevelEntityId(pub(crate) String);

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct LevelEntityType(pub(crate) String);

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AnimationCatalog(pub(crate) HashMap<String, Vec<String>>);

#[derive(Component)]
pub(crate) struct NpcMoving;

pub(crate) fn spawn_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity_definition: &EntityDefinition,
    entity_type: &EntityTypeDefinition,
    world_position: Vec3,
) -> Vec<String> {
    let sprite = sprite_for_entity(asset_server, entity_type);

    let mut entity_commands = commands.spawn((
        Name::new(format!("Entity:{}", entity_definition.id)),
        sprite,
        Transform::from_translation(world_position),
        SpawnedLevelEntity,
        LevelEntityId(entity_definition.id.clone()),
        LevelEntityType(entity_definition.entity_type.clone()),
        animation::AnimationState::default(),
        animation::AnimationPlayback::new(entity_type.animation_frame_seconds()),
    ));

    let mut warnings = Vec::new();
    let mut has_collision = false;
    let mut has_player = false;
    let mut has_moving = false;

    // Pre-compute animations once - insert separately to avoid early drop
    let normalized_animations = entity_type.normalized_animations();
    entity_commands.insert((
        AnimationCatalog(normalized_animations.clone()),
        animation::PreloadedAnimations::from_paths(asset_server, &normalized_animations),
    ));

    for component_name in &entity_type.components {
        match component_name.as_str() {
            "collision" => {
                has_collision = true;
                collision::insert(&mut entity_commands)
            }
            "doodad" => doodad::insert(&mut entity_commands),
            "exit" => exit::insert(&mut entity_commands),
            "floor" => floor::insert(&mut entity_commands),
            "npc" => npc::insert(&mut entity_commands),
            "hostile" => hostile::insert(&mut entity_commands),
            "moving" => {
                has_moving = true;
                moving::insert(&mut entity_commands, world_position.x)
            }
            "player" => {
                has_player = true;
                player::insert(&mut entity_commands)
            }
            other => warnings.push(format!(
                "{} references unknown component '{}'",
                entity_definition.id, other
            )),
        }
    }

    if let Some(hp) = entity_type.health {
        entity_commands.insert(health::Health::new(hp));
    }

    if let Some(dmg) = entity_type.damage {
        entity_commands.insert(health::Damage(dmg));
    }

    // Insert PlasmaAttack for player entities that define an attack range.
    if has_player {
        if let Some(range) = entity_type.attack_range {
            let dmg = entity_type.damage.unwrap_or(10);
            entity_commands.insert(player::PlasmaAttack::new(range, dmg));
        }
    }

    match hitbox::from_entity_type(entity_type) {
        Ok(polygon_hitbox) => {
            if has_player || has_moving {
                let precomputed_hitbox = hitbox::PrecomputedPlayerHitbox::from_polygon_hitbox(&polygon_hitbox);
                let collider = precomputed_hitbox.collider(false);

                entity_commands.insert((polygon_hitbox, precomputed_hitbox));

                let collision_layers = if has_moving && has_collision {
                    entity_commands.insert(NpcMoving);
                    CollisionLayers::new(
                        LayerMask(0b0010),
                        LayerMask(0b1101),
                    )
                } else {
                    CollisionLayers::default()
                };

                entity_commands.insert((
                    RigidBody::Dynamic,
                    collider,
                    LinearVelocity::ZERO,
                    LockedAxes::ROTATION_LOCKED,
                    CollidingEntities::default(),
                    collision_layers,
                ));
            } else if has_collision {
                let collider = hitbox::collider_from_points(polygon_hitbox.points.clone());

                entity_commands.insert(polygon_hitbox);
                entity_commands.insert((RigidBody::Static, collider));
            } else {
                entity_commands.insert(polygon_hitbox);
            }
        }
        Err(error) => {
            warnings.push(format!("{} has invalid hitbox: {error}", entity_definition.id));
        }
    }

    warnings
}

fn sprite_for_entity(asset_server: &AssetServer, entity_type: &EntityTypeDefinition) -> Sprite {
    match entity_type.default_animation_path() {
        Some(path) => {
            let mut sprite = Sprite::from_image(asset_server.load(path));
            sprite.custom_size = Some(entity_type.size());
            sprite
        }
        None => placeholder_sprite(color_for_components(&entity_type.components), entity_type.size()),
    }
}

fn color_for_components(component_names: &[String]) -> Color {
    if component_names.iter().any(|name| name == "player") {
        return Color::srgb(0.2, 0.45, 1.0);
    }

    if component_names.iter().any(|name| name == "hostile") {
        return Color::srgb(0.75, 0.18, 0.18);
    }

    if component_names.iter().any(|name| name == "npc") {
        return Color::srgb(0.3, 0.75, 0.35);
    }

    if component_names.iter().any(|name| name == "floor") {
        return Color::srgb(0.52, 0.35, 0.2);
    }

    if component_names.iter().any(|name| name == "doodad") {
        return Color::srgb(0.8, 0.7, 0.45);
    }

    Color::srgb(0.7, 0.7, 0.7)
}

fn placeholder_sprite(color: Color, size: Vec2) -> Sprite {
    Sprite {
        color,
        custom_size: Some(size),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_doodads_differently_from_unknown_entities() {
        assert_eq!(
            color_for_components(&["doodad".to_string()]),
            Color::srgb(0.8, 0.7, 0.45)
        );
        assert_eq!(
            color_for_components(&["something-else".to_string()]),
            Color::srgb(0.7, 0.7, 0.7)
        );
    }
}
