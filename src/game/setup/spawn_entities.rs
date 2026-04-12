use bevy::prelude::*;
use serde_json::Value as JsonValue;

use crate::game::components::{AutoMovement, Blocking, Collider, ColliderShape, ControlledMovement, Damageable, GameEntity, Gravity, Health, MovingPlatform, RigidBody, StateMachine};
use crate::game::components::auto_melee_attack::AutoMeleeAttack;
use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::controlled_melee_attack::ControlledMeleeAttack;
use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::state_machine::EntityState;
use crate::game::components::team::Team;
use crate::game::level::types::{
    CachedLevelDefinition, EntityTypeDefinition, LevelBounds, StateConfig, StateMachineConfig, PropValue,
};
use crate::game::runtime_components::AnimationConfig;
use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::tags::{DoodadTag, EnemyTag, EnvironmentTag, PlayerTag};

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

    let _bounds = level.bounds.clone().unwrap_or_default();

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
        let anim_cfg =
            AnimationConfig::new(state_cfg.animation.clone(), state_cfg.animation_frame_ms);
        let state_machine = StateMachine::new(parse_entity_state(initial_state_name));
        let collider = build_collider(state_cfg, sprite_w, sprite_h);

        // Generic component assignment: add only components explicitly listed
        // in the entity-type JSON (`entity_type.component`) or present in the
        // type's `components` object. This avoids implicit category-based
        // wiring and keeps component assignment fully data-driven.
        let mut ent_cmd = commands.spawn((sprite, transform, anim_cfg, state_machine, GameEntity));
        let mut assigned_components: Vec<String> = Vec::new();

        // Merge entity-type components with any per-entity overrides found in
        // the level JSON. Level entities may include a `components` object to
        // override default component values for that instance. We accept the
        // instance `components` as JSON (stored in `LevelEntity.properties` as
        // a serialized value) and merge keys — instance values overwrite type
        // defaults.
        let mut merged_components: std::collections::HashMap<String, serde_json::Value> =
            entity_type.components.clone().unwrap_or_default();

        if let Some(prop) = entity.properties.get("components") {
            match prop {
                PropValue::Other(s) | PropValue::String(s) => {
                    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(s) {
                        for (k, v) in map.into_iter() {
                            merged_components.insert(k, v);
                        }
                    } else {
                        tracing::warn!(id = %entity.id, "spawn_entities: could not parse entity-level 'components' override (expected object)");
                    }
                }
                _ => {
                    tracing::warn!(id = %entity.id, "spawn_entities: unexpected 'components' property type in level entity, expected object");
                }
            }
        }

        // Helper closures to read optional properties from the merged components map.
        let components_obj = if merged_components.is_empty() { None } else { Some(&merged_components) };
        let get_u64 = |key: &str| -> Option<u64> {
            components_obj
                .and_then(|obj| obj.get(key))
                .and_then(|v| v.as_u64())
        };
        let get_string = |key: &str| -> Option<String> {
            components_obj
                .and_then(|obj| obj.get(key))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };

        // Debug: if a moving_platform override is present in the merged map,
        // log the merged value so we can confirm level overrides are applied.
        if let Some(mp_val) = merged_components.get("moving_platform").or_else(|| merged_components.get("movingPlatform")).or_else(|| merged_components.get("moving-platform")) {
            tracing::info!(id = %entity.id, moving_platform = ?mp_val, "spawn_entities: merged moving_platform for entity");
        }

        // Insert collider if explicitly present in the components map or the
        // state defines a collider box.
        let has_collider = components_obj
            .map(|m| m.contains_key("collider"))
            .unwrap_or(false)
            || state_cfg.collider_box.is_some();
        if has_collider {
            ent_cmd.insert(collider.clone());
            assigned_components.push("Collider".to_string());
        }

        // Iterate declared component keys in the entity-type `components` map.
        let comp_keys: Vec<String> = components_obj
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        for comp in comp_keys {
            let comp_obj = components_obj.and_then(|m| m.get(&comp));
            match comp.to_ascii_lowercase().as_str() {
                "health" => {
                    let health_comp = Health::default().override_from_json(comp_obj);
                    ent_cmd.insert(health_comp);
                    assigned_components.push("Health".to_string());
                }
                "controlledmovement" | "controlled_movement" => {
                    let cm = ControlledMovement::default().override_from_json(comp_obj);
                    ent_cmd.insert(cm);
                    assigned_components.push("ControlledMovement".to_string());
                }
                "automovement" | "auto_movement" => {
                    let am = AutoMovement::default().override_from_json(comp_obj);
                    ent_cmd.insert(am);
                    assigned_components.push("AutoMovement".to_string());
                }
                "movingplatform" | "moving_platform" => {
                    let mp = MovingPlatform::default().override_from_json(comp_obj);
                    ent_cmd.insert(mp);
                    assigned_components.push("MovingPlatform".to_string());
                }
                "rigidbody" | "rigid_body" => {
                    let rb = RigidBody::default().override_from_json(comp_obj);
                    ent_cmd.insert(rb);
                    assigned_components.push("RigidBody".to_string());
                }
                "gravity" => {
                    let g = Gravity::default().override_from_json(comp_obj);
                    ent_cmd.insert(g);
                    assigned_components.push("Gravity".to_string());
                }
                "blocking" => {
                    let b = Blocking::default().override_from_json(comp_obj);
                    ent_cmd.insert(b);
                    assigned_components.push("Blocking".to_string());
                }
                "controlled_range_attack" | "controlledrangeattack" | "controlled_range" => {
                    let cra = ControlledRangeAttack::default().override_from_json(comp_obj);
                    ent_cmd.insert(cra);
                    assigned_components.push("ControlledRangeAttack".to_string());
                }
                "auto_range_attack" | "autorangeattack" | "auto_range" => {
                    let ara = AutoRangeAttack::default().override_from_json(comp_obj);
                    ent_cmd.insert(ara);
                    assigned_components.push("AutoRangeAttack".to_string());
                }
                "auto_melee_attack" | "automeleeattack" | "auto_melee" => {
                    let ama = AutoMeleeAttack::default().override_from_json(comp_obj);
                    ent_cmd.insert(ama);
                    assigned_components.push("AutoMeleeAttack".to_string());
                }
                "controlled_melee_attack" | "controlledmeleeattack" | "controlled_melee" => {
                    let cma = ControlledMeleeAttack::default().override_from_json(comp_obj);
                    ent_cmd.insert(cma);
                    assigned_components.push("ControlledMeleeAttack".to_string());
                }
                "damageable" => {
                    let d = Damageable::default().override_from_json(comp_obj);
                    ent_cmd.insert(d);
                    assigned_components.push("Damageable".to_string());
                }
                "team" => {
                    let team = Team::default().override_from_json(comp_obj);
                    ent_cmd.insert(team);
                    assigned_components.push("Team".to_string());
                }
                other => {
                    // Unknown component names are currently ignored; designers
                    // must reference existing runtime components by name in JSON.
                    tracing::warn!(id = %entity.id, comp = %other, "spawn_entities: unknown component in entity_type.component, skipping");
                }
            }
        }
        // Assign tag components strictly from `category_tag` (no tag logic
        // from the components map). This ensures tags are deterministic and
        // authored via the high-level category field.
        if let Some(cat) = entity_type.category_tag.as_ref() {
            match cat.to_ascii_lowercase().as_str() {
                "player" => {
                    ent_cmd.insert(PlayerTag);
                    assigned_components.push("PlayerTag".to_string());
                }
                "enemy" => {
                    ent_cmd.insert(EnemyTag);
                    assigned_components.push("EnemyTag".to_string());
                }
                "environment" | "movingplatform" => {
                    ent_cmd.insert(EnvironmentTag);
                    assigned_components.push("EnvironmentTag".to_string());
                }
                "doodad" => {
                    ent_cmd.insert(DoodadTag);
                    assigned_components.push("DoodadTag".to_string());
                }
                _ => {}
            }
        }

        // Preserve the level JSON id and entity_type at runtime so debug
        // overlays and other systems can reference them.
        ent_cmd.insert(SpawnedLevelEntity {
            id: entity.id.clone(),
            entity_type: entity.entity_type.clone(),
            layer: entity.layer.clone(),
        });

        tracing::info!(id = %entity.id, x, y, assigned_components = ?assigned_components, "Spawned entity with data-driven components");
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
