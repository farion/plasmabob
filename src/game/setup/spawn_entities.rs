use bevy::prelude::*;

use crate::game::gameplay::components::{
    AnimationConfig, AutoMovement, Blocking, Collider, ColliderShape, ControlledMovement,
    DoodadTag, EnemyTag, EnvironmentTag, GameEntity, Gravity, Health, PlayerTag, RigidBody,
    StateMachine,
};
use crate::game::level::types::{
    CachedLevelDefinition, EntityTypeDefinition, LevelBounds, StateConfig, StateMachineConfig,
};
use crate::game::gameplay::components::state_machine::EntityState;

/// Spawns all entities defined in the level at their configured world positions,
/// with the correct initial animation state and gameplay components attached.
///
/// Coordinate mapping
/// ──────────────────
/// Level JSON uses a bottom-left origin (x right, y up). Entity positions
/// mark the bottom-left corner of the sprite. Bevy sprites are centred on
/// their `Transform`, so we offset by half the sprite size:
///
///   bevy_x = entity.x + sprite_width  / 2
///   bevy_y = entity.y + sprite_height / 2
pub fn spawn_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cached: Res<CachedLevelDefinition>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("spawn_entities: no level loaded, skipping entity spawn");
        return;
    };

    let entities = match level.entities.as_deref() {
        Some(e) if !e.is_empty() => e,
        _ => {
            tracing::debug!("spawn_entities: level has no entities");
            return;
        }
    };

    let bounds = level.bounds.clone().unwrap_or_default();

    for entity in entities {
        let Some(entity_type) = cached.entity_types.get(&entity.entity_type) else {
            tracing::warn!(
                id = %entity.id,
                entity_type = %entity.entity_type,
                "spawn_entities: unknown entity type, skipping"
            );
            continue;
        };

        let Some(sm_cfg) = entity_type.state_machine_config() else {
            tracing::warn!(
                id = %entity.id,
                entity_type = %entity.entity_type,
                "spawn_entities: no state_machine config found, skipping"
            );
            continue;
        };

        let initial_state_name = sm_cfg.initial_state.as_str();
        let Some(state_cfg) = sm_cfg.states.get(initial_state_name) else {
            tracing::warn!(
                id = %entity.id,
                state = %initial_state_name,
                "spawn_entities: initial state not found in states map, skipping"
            );
            continue;
        };

        let sprite_w = entity_type.width.unwrap_or(128) as f32;
        let sprite_h = entity_type.height.unwrap_or(128) as f32;

        // Place transform at sprite centre (level coords use bottom-left).
        let x = entity.x + sprite_w / 2.0;
        let y = entity.y + sprite_h / 2.0;
        let z = entity.z_index;

        let first_frame = state_cfg.animation.first().cloned().unwrap_or_default();

        let sprite = Sprite {
            image: asset_server.load(&first_frame),
            custom_size: Some(Vec2::new(sprite_w, sprite_h)),
            ..default()
        };
        let transform = Transform::from_xyz(x, y, z);
        let anim_cfg = AnimationConfig::new(state_cfg.animation.clone(), state_cfg.animation_frame_ms);
        let state_machine = StateMachine::new(parse_entity_state(initial_state_name));
        let collider = build_collider(state_cfg, sprite_w, sprite_h);

        // Determine category from the first entry in `component`.
        let category = entity_type
            .component
            .first()
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        match category.as_str() {
            "player" => {
                let hp = entity_type.health.unwrap_or(100) as i32;
                commands.spawn((
                    sprite,
                    transform,
                    anim_cfg,
                    state_machine,
                    Health::new(hp),
                    ControlledMovement::default(),
                    RigidBody::default(),
                    Gravity::default(),
                    collider,
                    PlayerTag,
                    GameEntity,
                ));
                tracing::debug!(id = %entity.id, x, y, "Spawned player");
            }
            "enemy" => {
                let hp = entity_type.health.unwrap_or(10) as i32;
                commands.spawn((
                    sprite,
                    transform,
                    anim_cfg,
                    state_machine,
                    Health::new(hp),
                    AutoMovement::default(),
                    RigidBody::default(),
                    Gravity::default(),
                    collider,
                    EnemyTag,
                    GameEntity,
                ));
                tracing::debug!(id = %entity.id, x, y, "Spawned enemy");
            }
            "environment" | "movingplatform" => {
                commands.spawn((
                    sprite,
                    transform,
                    anim_cfg,
                    state_machine,
                    Blocking,
                    collider,
                    EnvironmentTag,
                    GameEntity,
                ));
                tracing::debug!(id = %entity.id, x, y, category = %category, "Spawned environment");
            }
            _ => {
                // Doodads, exits, pickups and any other decorative / trigger entities.
                commands.spawn((
                    sprite,
                    transform,
                    anim_cfg,
                    state_machine,
                    DoodadTag,
                    GameEntity,
                ));
                tracing::debug!(id = %entity.id, x, y, category = %category, "Spawned doodad");
            }
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Map a state name string to the typed `EntityState` enum.
fn parse_entity_state(s: &str) -> EntityState {
    match s.to_ascii_lowercase().as_str() {
        "idle" => EntityState::Idle,
        "moving" | "walking" | "running" => EntityState::Moving,
        "jumping" => EntityState::Jumping,
        "falling" => EntityState::Falling,
        "damaged" | "hit" => EntityState::Damaged,
        "dying" => EntityState::Dying,
        "dead" => EntityState::Dead,
        "melee_attacking" | "meleeattacking" => EntityState::MeleeAttacking,
        "range_attacking" | "rangeattacking" => EntityState::RangeAttacking,
        "crouching" => EntityState::Crouching,
        _ => {
            tracing::warn!(state = %s, "Unknown entity state string, defaulting to Idle");
            EntityState::Idle
        }
    }
}

/// Build a `Collider` from a state's `collider_box` (pixel coords in image
/// space, origin top-left). Falls back to the full sprite rectangle when the
/// collider box is absent.
///
/// Conversion from image space to entity-local Bevy space:
///   local_x = pixel_x − sprite_w / 2          (shift origin to centre)
///   local_y = sprite_h / 2 − pixel_y          (flip Y axis)
fn build_collider(state_cfg: &StateConfig, sprite_w: f32, sprite_h: f32) -> Collider {
    if let Some(pts) = &state_cfg.collider_box {
        if pts.len() >= 2 {
            let min_x = pts.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
            let max_x = pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
            let min_y = pts.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
            let max_y = pts.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

            let half_w = (max_x - min_x) / 2.0;
            let half_h = (max_y - min_y) / 2.0;

            // Centre of the box in image space → entity-local Bevy space.
            let cx_img = (min_x + max_x) / 2.0;
            let cy_img = (min_y + max_y) / 2.0;
            let offset_x = cx_img - sprite_w / 2.0;
            let offset_y = sprite_h / 2.0 - cy_img;

            return Collider {
                offset: Vec2::new(offset_x, offset_y),
                shape: ColliderShape::Rectangle {
                    half_extents: Vec2::new(half_w, half_h),
                },
                is_trigger: false,
            };
        }
    }

    // Fallback: full sprite bounding box centred at origin.
    Collider {
        offset: Vec2::ZERO,
        shape: ColliderShape::Rectangle {
            half_extents: Vec2::new(sprite_w / 2.0, sprite_h / 2.0),
        },
        is_trigger: false,
    }
}

