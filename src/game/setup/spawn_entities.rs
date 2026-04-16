use bevy::prelude::*;
use bevy::audio::AudioSource;
use serde_json::Value as JsonValue;

use crate::game::components::orientation::FacingDirection;
use crate::game::components::{AutoMovement, Blocking, Collider, ColliderShape, ControlledMovement, Damageable, GameEntity, Gravity, Health, MovingPlatform, Orientation, RigidBody, StateMachine};
use crate::game::components::auto_melee_attack::AutoMeleeAttack;
use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::controlled_melee_attack::ControlledMeleeAttack;
use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::state_machine::EntityState;
use crate::game::components::team::Team;
use crate::game::level::types::{
    CachedLevelDefinition, EntityTypeDefinition, LevelBounds, StateConfig, StateMachineConfig, PropValue,
};
use crate::game::runtime_components::{AnimationConfig, SoundState};
use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::setup::collider_helper::build_collider_from_box;
use crate::game::setup::flip_utils::flip_entity_preserve_collider;
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::game::tags::{DoodadTag, EnemyTag, EnvironmentTag, PlayerTag};

/// Red fallback color used when a sprite is missing.
const MISSING_SPRITE_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

/// Spawns all entities defined in the level at their configured world positions,
/// with the correct initial animation state and gameplay components attached.
///
/// Sprite handles and state metadata are looked up from [`EntityTypeAssets`] when
/// available (pre-loaded by `LoadView`).  Falls back to on-demand `AssetServer::load`
/// when the resource is absent (e.g. dev hot-reload straight into GameView).
pub fn spawn_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cached: Res<CachedLevelDefinition>,
    entity_type_assets: Option<Res<EntityTypeAssets>>,
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
        // `LevelEntity.entity_type` is now the fully-typed `EntityTypeDefinition`.
        let entity_type = &entity.entity_type;

        let Some(sm_cfg) = entity_type.state_machine_config() else {
            tracing::warn!(
                id = %entity.id,
                entity_type = %entity.entity_type.key,
                "spawn_entities: no state_machine config found, skipping"
            );
            continue;
        };

        let initial_state_name = sm_cfg.initial_state.to_ascii_lowercase();
        let Some(state_cfg) = sm_cfg.states.get(&initial_state_name)
            .or_else(|| sm_cfg.states.get(&sm_cfg.initial_state)) else
        {
            tracing::warn!(
                id = %entity.id,
                state = %sm_cfg.initial_state,
                "spawn_entities: initial state not found in states map, skipping"
            );
            continue;
        };

        let sprite_w = entity_type.width.unwrap_or(128) as f32;
        let sprite_h = entity_type.height.unwrap_or(128) as f32;

        // Parse entity-type typed components and optional level-entity overrides.
        // We no longer convert the typed `ComponentsDef` into a HashMap. Call
        // sites must read the typed fields directly.
        let override_map: Option<std::collections::HashMap<String, serde_json::Value>> =
            match entity.properties.get("components") {
                Some(PropValue::Other(s)) | Some(PropValue::String(s)) => {
                    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(s) {
                        let mut hm = std::collections::HashMap::new();
                        for (k, v) in map.into_iter() { hm.insert(k, v); }
                        Some(hm)
                    } else {
                        tracing::warn!(id = %entity.id, "spawn_entities: could not parse entity-level 'components' override (expected object)");
                        None
                    }
                }
                Some(_) => {
                    tracing::warn!(id = %entity.id, "spawn_entities: unexpected 'components' property type in level entity, expected object");
                    None
                }
                None => None,
            };

        // Helper: retrieve a per-entity override (raw JSON) if present.
        let comps_def = entity_type.components.as_ref();
        let get_override = |key: &str| -> Option<&serde_json::Value> {
            override_map.as_ref().and_then(|m| m.get(key))
        };

        // Orientation is always present on every entity (defaults to Right / 0°)
        // and can be overridden via JSON key `orientation` or provided by the
        // entity-type's typed ComponentsDef.
        let orientation = if let Some(ov) = get_override("orientation") {
            Orientation::default().override_from_json(Some(ov))
        } else if let Some(c) = comps_def.and_then(|c| c.orientation.as_ref()) {
            c.clone()
        } else {
            Orientation::default()
        };

        // Place transform at sprite centre (level coords use bottom-left).
        let x = entity.x + sprite_w / 2.0;
        let y = entity.y + sprite_h / 2.0;
        let z = entity.z_index;

        // ── Resolve animation frames ──────────────────────────────────────────
        let (frames, sprite_image, sprite_color) = if let Some(ref eta) = entity_type_assets {
            if let Some(state_assets) = eta.get_state(&entity.entity_type.key, &initial_state_name) {
                let first = state_assets.frames.first().cloned();
                if let Some(h) = first {
                    (state_assets.frames.clone(), h, Color::WHITE)
                } else {
                    // Missing sprite → red fallback
                    tracing::warn!(
                        entity_type = %entity.entity_type.key,
                        state = %initial_state_name,
                        "spawn_entities: no frames in EntityTypeAssets, using red fallback"
                    );
                    (vec![], asset_server.load(""), MISSING_SPRITE_COLOR)
                }
            } else {
                // State not in cache, fall back
                build_frames_from_cfg(state_cfg, &asset_server)
            }
        } else {
            // No EntityTypeAssets available → load on the fly
            build_frames_from_cfg(state_cfg, &asset_server)
        };

        let anim_cfg = AnimationConfig::new(frames, state_cfg.animation_frame_ms);

        let desired_flip = matches!(orientation.facing, FacingDirection::Left);

        let mut transform = Transform::from_xyz(x, y, z);
        let mut sprite = Sprite {
            image: sprite_image.clone(),
            color: sprite_color,
            custom_size: Some(Vec2::new(sprite_w, sprite_h)),
            flip_x: desired_flip,
            ..default()
        };


        // Generic component assignment: add only components explicitly listed
        // in the entity-type JSON (`entity_type.component`) or present in the
        // type's `components` object.
        let mut ent_cmd = commands.spawn((sprite, transform, anim_cfg, GameEntity));
        let mut assigned_components: Vec<String> = Vec::new();

        ent_cmd.insert(orientation);


        // Instantiate components from typed `ComponentsDef` or per-entity overrides.
        // Health
        if let Some(ov) = get_override("health") {
            let health_comp = Health::default().override_from_json(Some(ov));
            ent_cmd.insert(health_comp);
            assigned_components.push("Health".to_string());
        } else if let Some(h) = comps_def.and_then(|c| c.health.clone()) {
            ent_cmd.insert(h);
            assigned_components.push("Health".to_string());
        }

        // ControlledMovement
        if let Some(ov) = get_override("controlled_movement") {
            let cm = ControlledMovement::default().override_from_json(Some(ov));
            ent_cmd.insert(cm);
            assigned_components.push("ControlledMovement".to_string());
        } else if let Some(cm) = comps_def.and_then(|c| c.controlled_movement.clone()) {
            ent_cmd.insert(cm);
            assigned_components.push("ControlledMovement".to_string());
        }

        // AutoMovement
        if let Some(ov) = get_override("auto_movement") {
            let am = AutoMovement::default().override_from_json(Some(ov));
            ent_cmd.insert(am);
            assigned_components.push("AutoMovement".to_string());
        } else if let Some(am) = comps_def.and_then(|c| c.auto_movement.clone()) {
            ent_cmd.insert(am);
            assigned_components.push("AutoMovement".to_string());
        }

        // MovingPlatform
        if let Some(ov) = get_override("moving_platform") {
            let mp = MovingPlatform::default().override_from_json(Some(ov));
            ent_cmd.insert(mp);
            assigned_components.push("MovingPlatform".to_string());
        } else if let Some(mp) = comps_def.and_then(|c| c.moving_platform.clone()) {
            ent_cmd.insert(mp);
            assigned_components.push("MovingPlatform".to_string());
        }

        // RigidBody
        if let Some(ov) = get_override("rigid_body") {
            let rb = RigidBody::default().override_from_json(Some(ov));
            ent_cmd.insert(rb);
            assigned_components.push("RigidBody".to_string());
        } else if let Some(rb) = comps_def.and_then(|c| c.rigid_body.clone()) {
            ent_cmd.insert(rb);
            assigned_components.push("RigidBody".to_string());
        }

        // Gravity
        if let Some(ov) = get_override("gravity") {
            let g = Gravity::default().override_from_json(Some(ov));
            ent_cmd.insert(g);
            assigned_components.push("Gravity".to_string());
        } else if let Some(g) = comps_def.and_then(|c| c.gravity.clone()) {
            ent_cmd.insert(g);
            assigned_components.push("Gravity".to_string());
        }

        // Blocking
        if let Some(ov) = get_override("blocking") {
            let b = Blocking::default().override_from_json(Some(ov));
            ent_cmd.insert(b);
            assigned_components.push("Blocking".to_string());
        } else if let Some(b) = comps_def.and_then(|c| c.blocking.clone()) {
            ent_cmd.insert(b);
            assigned_components.push("Blocking".to_string());
        }

        // ControlledRangeAttack
        if let Some(ov) = get_override("controlled_range_attack") {
            let cra = ControlledRangeAttack::default().override_from_json(Some(ov));
            ent_cmd.insert(cra);
            assigned_components.push("ControlledRangeAttack".to_string());
        } else if let Some(cra) = comps_def.and_then(|c| c.controlled_range_attack.clone()) {
            ent_cmd.insert(cra);
            assigned_components.push("ControlledRangeAttack".to_string());
        }

        // AutoRangeAttack
        if let Some(ov) = get_override("auto_range_attack") {
            let ara = AutoRangeAttack::default().override_from_json(Some(ov));
            ent_cmd.insert(ara);
            assigned_components.push("AutoRangeAttack".to_string());
        } else if let Some(ara) = comps_def.and_then(|c| c.auto_range_attack.clone()) {
            ent_cmd.insert(ara);
            assigned_components.push("AutoRangeAttack".to_string());
        }

        // AutoMeleeAttack
        if let Some(ov) = get_override("auto_melee_attack") {
            let ama = AutoMeleeAttack::default().override_from_json(Some(ov));
            ent_cmd.insert(ama);
            assigned_components.push("AutoMeleeAttack".to_string());
        } else if let Some(ama) = comps_def.and_then(|c| c.auto_melee_attack.clone()) {
            ent_cmd.insert(ama);
            assigned_components.push("AutoMeleeAttack".to_string());
        }

        // ControlledMeleeAttack
        if let Some(ov) = get_override("controlled_melee_attack") {
            let cma = ControlledMeleeAttack::default().override_from_json(Some(ov));
            ent_cmd.insert(cma);
            assigned_components.push("ControlledMeleeAttack".to_string());
        } else if let Some(cma) = comps_def.and_then(|c| c.controlled_melee_attack.clone()) {
            ent_cmd.insert(cma);
            assigned_components.push("ControlledMeleeAttack".to_string());
        }

        // Damageable
        if let Some(ov) = get_override("damageable") {
            let d = Damageable::default().override_from_json(Some(ov));
            ent_cmd.insert(d);
            assigned_components.push("Damageable".to_string());
        } else if let Some(d) = comps_def.and_then(|c| c.damageable.clone()) {
            ent_cmd.insert(d);
            assigned_components.push("Damageable".to_string());
        }

        // Team
        if let Some(ov) = get_override("team") {
            let team = Team::default().override_from_json(Some(ov));
            ent_cmd.insert(team);
            assigned_components.push("Team".to_string());
        } else if let Some(team) = comps_def.and_then(|c| c.team.clone()) {
            ent_cmd.insert(team);
            assigned_components.push("Team".to_string());
        }

        // Create StateMachine only when explicitly declared in the entity's
        // components. Use the entity-type state's configured initial state.
        if get_override("state_machine").is_some() || comps_def.is_some() {
            // Build runtime StateMachine from the typed state machine config
            // (preferred: components.extra["state_machine"]; fallback: top-level config).
            let sm_cfg = entity_type.state_machine_config();
            if let Some(cfg) = sm_cfg {
                let mut sm = StateMachine::from_config(&cfg);
                ent_cmd.insert(sm.clone());
                let initial_state_enum = parse_entity_state(&cfg.initial_state);
                let ss = SoundState::new(initial_state_enum);
                ent_cmd.insert(ss);
                assigned_components.push("StateMachine".to_string());
                assigned_components.push("SoundState".to_string());
            } else {
                // fallback: create default state machine using the initial state name
                let initial_state_enum = parse_entity_state(&initial_state_name);
                let sm = StateMachine::new(initial_state_enum);
                ent_cmd.insert(sm);
                let ss = SoundState::new(initial_state_enum);
                ent_cmd.insert(ss);
                assigned_components.push("StateMachine".to_string());
                assigned_components.push("SoundState".to_string());
            }
        }

        // Collider
        if get_override("collider").is_some() || comps_def.and_then(|c| c.collider.as_ref()).is_some() {
            let mut col = build_collider_from_box(state_cfg.collider_box.as_deref(), sprite_w, sprite_h);
            if desired_flip {
                let cx = col.offset.x;
                let transform_x = x + 2.0 * cx;
                col.offset.x = -cx;
                ent_cmd.insert(col.clone());
                ent_cmd.insert(Transform::from_xyz(transform_x, y, z));
                let spr = Sprite {
                    image: sprite_image.clone(),
                    color: sprite_color,
                    custom_size: Some(Vec2::new(sprite_w, sprite_h)),
                    flip_x: true,
                    ..default()
                };
                ent_cmd.insert(spr);
            } else {
                ent_cmd.insert(col.clone());
            }
            assigned_components.push("Collider".to_string());
        }

        // Warn about unknown override keys that are not handled explicitly.
        if let Some(ov) = override_map.as_ref() {
            for k in ov.keys() {
                let canonical = k.to_ascii_lowercase();
                let known = matches!(canonical.as_str(),
                    "health" | "controlled_movement" | "automovement" | "auto_movement" | "moving_platform" | "movingplatform" |
                    "rigid_body" | "rigidbody" | "gravity" | "blocking" | "controlled_range_attack" | "controlledrangeattack" |
                    "auto_range_attack" | "autorangeattack" | "auto_melee_attack" | "automeleeattack" | "controlled_melee_attack" | "controlledmeleeattack" |
                    "damageable" | "team" | "orientation" | "state_machine" | "statemachine" | "collider"
                );
                if !known {
                    tracing::warn!(id = %entity.id, comp = %k, "spawn_entities: unknown component key in entity-level components override");
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
            entity_type: entity.entity_type.key.clone(),
            layer: entity.layer.clone(),
        });

        tracing::info!(id = %entity.id, entity_type = %entity.entity_type.key, x, y, assigned_components = ?assigned_components,
            "Spawned {}", entity.name.clone());
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build animation frames from a `StateConfig` by loading each path via the AssetServer.
/// Returns (handles, first_handle, color).
fn build_frames_from_cfg(
    state_cfg: &StateConfig,
    asset_server: &AssetServer,
) -> (Vec<Handle<Image>>, Handle<Image>, Color) {
    let frames: Vec<Handle<Image>> = state_cfg.animation.iter()
        .map(|p| asset_server.load::<Image>(p))
        .collect();
    let first = frames.first().cloned().unwrap_or_else(|| asset_server.load(""));
    (frames, first, Color::WHITE)
}

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
