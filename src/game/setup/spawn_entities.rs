use bevy::prelude::*;

use crate::game::components::orientation::FacingDirection;
use crate::game::components::{AutoMovement, Blocking, CollectibleEffect, ControlledMovement, Damageable, GameEntity, Gravity, Health, MovingPlatform, Orientation, RigidBody, StateMachine};
// component configs are parsed into `ComponentsDef` and consumed via each component's `override_from_config`
use crate::game::components::auto_melee_attack::AutoMeleeAttack;
use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::controlled_melee_attack::ControlledMeleeAttack;
use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::state_machine::EntityState;
use crate::game::components::team::Team;
use crate::game::level::types::{
    CachedLevelDefinition, StateConfig,
};
use crate::game::runtime_components::{AnimationConfig, SoundState};
use crate::game::runtime_components::SpawnedLevelEntity;
use crate::game::setup::collider_helper::build_collider_from_box;
// flip_utils was unused here; removed to silence warnings
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::game::tags::{DoodadTag, EnemyTag, EnvironmentTag, PlayerTag, CollectibleTag};
use crate::helper::active_character::ActiveCharacter;
use crate::helper::asset_io::load_character_asset;

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
    active_character: Res<ActiveCharacter>,
    cached: Res<CachedLevelDefinition>,
    entity_type_assets: Option<Res<EntityTypeAssets>>,
) {
    let Some(level) = &cached.level else {
        tracing::warn!("spawn_entities: no level loaded, skipping entity spawn");
        return;
    };

    let entities = level.entities.as_slice();
    if entities.is_empty() {
        tracing::debug!("spawn_entities: level has no entities");
        return;
    }

    let _bounds = level.bounds.clone().unwrap_or_default();

    for entity in entities {
        let entity_type_key = &entity.entity_type;
        let Some(entity_type) = cached.entity_types.get(entity_type_key) else {
            tracing::warn!(id = %entity.id, entity_type = %entity_type_key, "spawn_entities: unknown entity type, skipping");
            continue;
        };

        let Some(sm_cfg) = entity_type.state_machine_config() else {
            tracing::warn!(
                id = %entity.id,
                entity_type = %entity_type_key,
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

        let sprite_w = entity_type.width.unwrap_or(128.0);
        let sprite_h = entity_type.height.unwrap_or(128.0);

        // Typed component defs from entity-type and optional per-entity
        // ComponentsDef parsed from level JSON (if present).
        let entity_type_comps = entity_type.components.as_ref();
        let level_comps = entity.components.clone();

        // Orientation component: construct from defaults and apply typed
        // overrides coming from the entity-type and/or per-entity components.
        // Use the component's `override_from_config` helper which accepts
        // typed `OrientationConfig` references.
        let orientation = Orientation::default().override_from_config(
            entity_type_comps.and_then(|c| c.orientation.as_ref()),
            level_comps.as_ref().and_then(|c| c.orientation.as_ref()),
        );

        // Place transform at sprite centre (level coords use bottom-left).
        let x = entity.x + sprite_w / 2.0;
        let y = entity.y + sprite_h / 2.0;
        let z = entity.z_index.unwrap_or(0.0);

        // ── Resolve animation frames ──────────────────────────────────────────
        let (frames, sprite_image, sprite_color) = if let Some(ref eta) = entity_type_assets {
            if let Some(state_assets) = eta.get_state(entity_type_key, &initial_state_name) {
                let first = state_assets.frames.first().cloned();
                if let Some(h) = first {
                    (state_assets.frames.clone(), h, Color::WHITE)
                } else {
                    // Missing sprite → red fallback
                    tracing::warn!(
                        entity_type = %entity_type_key,
                        state = %initial_state_name,
                        "spawn_entities: no frames in EntityTypeAssets, using red fallback"
                    );
                    (
                        vec![],
                        load_character_asset::<Image>(&asset_server, "", *active_character),
                        MISSING_SPRITE_COLOR,
                    )
                }
            } else {
                // State not in cache, fall back
                build_frames_from_cfg(state_cfg, &asset_server, *active_character)
            }
        } else {
            // No EntityTypeAssets available → load on the fly
            build_frames_from_cfg(state_cfg, &asset_server, *active_character)
        };

        let anim_cfg = AnimationConfig::new(frames, state_cfg.animation_frame_ms);

        let desired_flip = matches!(orientation.facing, FacingDirection::Left);

        let transform = Transform::from_xyz(x, y, z);
        let sprite = Sprite {
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

        // Insert optional components only when declared in the entity-type or per-entity components
        if entity_type_comps.and_then(|c| c.health.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.health.as_ref()).is_some() {
            ent_cmd.insert(Health::default().override_from_config(
                entity_type_comps.and_then(|c| c.health.as_ref()),
                level_comps.as_ref().and_then(|c| c.health.as_ref())));
            assigned_components.push("Health".to_string());
        }

        if entity_type_comps.and_then(|c| c.controlled_movement.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.controlled_movement.as_ref()).is_some() {
            ent_cmd.insert(ControlledMovement::default().override_from_config(
                entity_type_comps.and_then(|c| c.controlled_movement.as_ref()),
                level_comps.as_ref().and_then(|c| c.controlled_movement.as_ref())));
            assigned_components.push("ControlledMovement".to_string());
        }

        if entity_type_comps.and_then(|c| c.auto_movement.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.auto_movement.as_ref()).is_some() {
            ent_cmd.insert(AutoMovement::default().override_from_config(
                entity_type_comps.and_then(|c| c.auto_movement.as_ref()),
                level_comps.as_ref().and_then(|c| c.auto_movement.as_ref())));
            assigned_components.push("AutoMovement".to_string());
        }

        if entity_type_comps.and_then(|c| c.moving_platform.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.moving_platform.as_ref()).is_some() {
            ent_cmd.insert(MovingPlatform::default().override_from_config(
                entity_type_comps.and_then(|c| c.moving_platform.as_ref()),
                level_comps.as_ref().and_then(|c| c.moving_platform.as_ref())));
            assigned_components.push("MovingPlatform".to_string());
        }

        if entity_type_comps.and_then(|c| c.rigid_body.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.rigid_body.as_ref()).is_some() {
            ent_cmd.insert(RigidBody::default().override_from_config(
                entity_type_comps.and_then(|c| c.rigid_body.as_ref()),
                level_comps.as_ref().and_then(|c| c.rigid_body.as_ref())));
            assigned_components.push("RigidBody".to_string());
        }

        if entity_type_comps.and_then(|c| c.gravity.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.gravity.as_ref()).is_some() {
            ent_cmd.insert(Gravity::default().override_from_config(
                entity_type_comps.and_then(|c| c.gravity.as_ref()),
                level_comps.as_ref().and_then(|c| c.gravity.as_ref())));
            assigned_components.push("Gravity".to_string());
        }

        if entity_type_comps.and_then(|c| c.blocking.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.blocking.as_ref()).is_some() {
            ent_cmd.insert(Blocking::default());
            assigned_components.push("Blocking".to_string());
        }

        if entity_type_comps.and_then(|c| c.controlled_range_attack.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.controlled_range_attack.as_ref()).is_some() {
            ent_cmd.insert(ControlledRangeAttack::default().override_from_config(
                entity_type_comps.and_then(|c| c.controlled_range_attack.as_ref()),
                level_comps.as_ref().and_then(|c| c.controlled_range_attack.as_ref())));
            assigned_components.push("ControlledRangeAttack".to_string());
        }

        if entity_type_comps.and_then(|c| c.auto_range_attack.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.auto_range_attack.as_ref()).is_some() {
            ent_cmd.insert(AutoRangeAttack::default().override_from_config(
                entity_type_comps.and_then(|c| c.auto_range_attack.as_ref()),
                level_comps.as_ref().and_then(|c| c.auto_range_attack.as_ref())));
            assigned_components.push("AutoRangeAttack".to_string());
        }

        if entity_type_comps.and_then(|c| c.auto_melee_attack.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.auto_melee_attack.as_ref()).is_some() {
            ent_cmd.insert(AutoMeleeAttack::default().override_from_config(
                entity_type_comps.and_then(|c| c.auto_melee_attack.as_ref()),
                level_comps.as_ref().and_then(|c| c.auto_melee_attack.as_ref())));
            assigned_components.push("AutoMeleeAttack".to_string());
        }

        if entity_type_comps.and_then(|c| c.controlled_melee_attack.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.controlled_melee_attack.as_ref()).is_some() {
            ent_cmd.insert(ControlledMeleeAttack::default().override_from_config(
                entity_type_comps.and_then(|c| c.controlled_melee_attack.as_ref()),
                level_comps.as_ref().and_then(|c| c.controlled_melee_attack.as_ref())));
            assigned_components.push("ControlledMeleeAttack".to_string());
        }

        if entity_type_comps.and_then(|c| c.damageable.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.damageable.as_ref()).is_some() {
            ent_cmd.insert(Damageable::default().override_from_config(
                entity_type_comps.and_then(|c| c.damageable.as_ref()),
                level_comps.as_ref().and_then(|c| c.damageable.as_ref())));
            assigned_components.push("Damageable".to_string());
        }

        if entity_type_comps.and_then(|c| c.team.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.team.as_ref()).is_some() {
            ent_cmd.insert(Team::default().override_from_config(
                entity_type_comps.and_then(|c| c.team.as_ref()),
                level_comps.as_ref().and_then(|c| c.team.as_ref())));
            assigned_components.push("Team".to_string());
        }

        if entity_type_comps.and_then(|c| c.collectible_effect.as_ref()).is_some() || level_comps.as_ref().and_then(|c| c.collectible_effect.as_ref()).is_some(){
            ent_cmd.insert(CollectibleEffect::default().override_from_config(
                entity_type_comps.and_then(|c| c.collectible_effect.as_ref()),
                level_comps.as_ref().and_then(|c| c.collectible_effect.as_ref())));
            assigned_components.push("CollectibleEffect".to_string());
        }




        // Create StateMachine only when explicitly declared in the entity's
        // components. Use the entity-type state's configured initial state.
        if level_comps.is_some() || entity_type_comps.is_some() {
            // Build runtime StateMachine from the typed state machine config
            // (preferred: components.extra["state_machine"]; fallback: top-level config).
            let sm_cfg = entity_type.state_machine_config();
            if let Some(cfg) = sm_cfg {
                let sm = StateMachine::from_config(&cfg);
                ent_cmd.insert(sm.clone());
                let initial_state_enum = parse_entity_state(&cfg.initial_state);
                let ss = SoundState::new(initial_state_enum);
                ent_cmd.insert(ss);
                assigned_components.push("StateMachine".to_string());
                assigned_components.push("SoundState".to_string());
            }
        }

        // Collider
        if level_comps.as_ref().and_then(|c| c.collider.as_ref()).is_some() || entity_type_comps.and_then(|c| c.collider.as_ref()).is_some() {
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

        // (no unknown-override warnings: per-entity components are typed)
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
                "collectible" => {
                    ent_cmd.insert(CollectibleTag);
                    assigned_components.push("CollectibleTag".to_string());
                }
                _ => {}
            }
        }

        // Preserve the level JSON id and entity_type at runtime so debug
        // overlays and other systems can reference them.
        ent_cmd.insert(SpawnedLevelEntity {
            id: entity.id.clone(),
            entity_type: entity_type_key.clone(),
            layer: entity.layer.clone().unwrap_or_else(|| "gameplay".to_string()),
        });

        tracing::info!(id = %entity.id, entity_type = %entity_type_key, x, y, assigned_components = ?assigned_components,
            "Spawned {}", entity.name.clone().unwrap_or_else(|| entity.id.clone()));
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build animation frames from a `StateConfig` by loading each path via the AssetServer.
/// Returns (handles, first_handle, color).
fn build_frames_from_cfg(
    state_cfg: &StateConfig,
    asset_server: &AssetServer,
    active_character: ActiveCharacter,
) -> (Vec<Handle<Image>>, Handle<Image>, Color) {
    let frames: Vec<Handle<Image>> = state_cfg.animation.iter()
        .map(|p| load_character_asset::<Image>(asset_server, p, active_character))
        .collect();
    let first = frames
        .first()
        .cloned()
        .unwrap_or_else(|| load_character_asset::<Image>(asset_server, "", active_character));
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
