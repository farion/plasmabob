use bevy::prelude::*;

use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::{
    AutoMovement, Collider, ControlledMovement, Damageable, EntityState, Gravity, Health,
    MovingPlatform, RigidBody, StateMachine,
};
use crate::game::runtime_components::{AnimationConfig, SpawnedLevelEntity};
use crate::game::setup::collider_helper::build_collider_from_box;
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::helper::key_bindings::KeyBindings;

/// Drives every entity's `StateMachine` state each frame based on movement, physics,
/// combat and damage signals.
///
/// Priority order (highest wins):
///   Dead > Dying > RangeAttacking > Damaged > Jumping / Crouching / Falling > Moving > Idle
///
/// Rules:
/// - **Dead**: terminal — no further transitions.
/// - **Dying**: waits `dying_duration_secs` (tracked via `state_time`) then becomes Dead.
/// - **Dying (new)**: triggered when `Health.is_dead()` is true.
/// - **RangeAttacking**: one-frame signal from `ControlledRangeAttack.just_fired`; cleared here.
/// - **Damaged**: `Damageable.damaged_timer > 0` (set by `projectile_collision_system`; ticked down here).
/// - **Jumping**: `ControlledMovement` entity, airborne, moving upward.
/// - **Crouching**: `ControlledMovement` entity, crouch key held.
/// - **Falling**: any entity with `Gravity`, airborne, moving downward.
/// - **Moving**: non-zero horizontal velocity (`ControlledMovement` / `AutoMovement`) or active `MovingPlatform`.
/// - **Idle**: default when no other condition applies.
pub fn state_machine_update_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    entity_type_assets: Option<Res<EntityTypeAssets>>,
    mut entities: Query<(
        &mut StateMachine,
        Option<&mut Damageable>,
        Option<&Health>,
        Option<&mut ControlledRangeAttack>,
        Option<&ControlledMovement>,
        Option<&AutoMovement>,
        Option<&MovingPlatform>,
        Option<&Gravity>,
        Option<&RigidBody>,
        Option<&mut Collider>,
        Option<&mut AnimationConfig>,
        Option<&mut Sprite>,
        Option<&SpawnedLevelEntity>,
    )>,
) {
    let dt = time.delta_secs();
    let crouch_key = key_bindings.crouch;

    for (
        mut sm,
        mut damageable,
        health,
        mut controlled_attack,
        controlled_movement,
        auto_movement,
        moving_platform,
        gravity,
        rigid_body,
        mut collider,
        mut anim_cfg,
        mut sprite,
        spawned,
    ) in &mut entities
    {
        // Always advance the state timer — `set_state()` resets it on transitions.
        sm.tick(dt);

        // Tick the damaged timer every frame so it counts down independently of state.
        if let Some(ref mut dmg) = damageable {
            dmg.damaged_timer = (dmg.damaged_timer - dt).max(0.0);
        }

        // --- Terminal state: Dead does not transition further ---
        if sm.is(EntityState::Dead) {
            continue;
        }

        // --- Dying: check whether the timer has elapsed and transition to Dead ---
        if sm.is(EntityState::Dying) {
            if sm.state_time >= sm.dying_duration_secs {
                apply_transition(&mut sm, EntityState::Dead, &entity_type_assets, spawned, &mut collider, &mut anim_cfg, &mut sprite);
            }
            continue;
        }

        // --- New death: health just reached zero ---
        if let Some(hp) = health {
            if hp.is_dead() {
                apply_transition(&mut sm, EntityState::Dying, &entity_type_assets, spawned, &mut collider, &mut anim_cfg, &mut sprite);
                continue;
            }
        }

        // --- Read and immediately clear the one-frame range-attack signal ---
        let is_range_attacking = controlled_attack
            .as_ref()
            .map(|a| a.just_fired)
            .unwrap_or(false);
        if let Some(ref mut atk) = controlled_attack {
            atk.just_fired = false;
        }

        // --- Damaged: timer was decremented above; check remaining time ---
        let is_damaged = damageable
            .as_ref()
            .map(|d| d.damaged_timer > 0.0)
            .unwrap_or(false);

        // --- Physics signals ---
        let grounded = gravity.map(|g| g.grounded).unwrap_or(true);
        let vel_y = rigid_body.map(|rb| rb.velocity.y).unwrap_or(0.0);
        let vel_x_abs = rigid_body.map(|rb| rb.velocity.x.abs()).unwrap_or(0.0);

        let is_jumping =
            controlled_movement.is_some() && !grounded && vel_y > f32::EPSILON;
        let is_crouching =
            controlled_movement.is_some() && keyboard.pressed(crouch_key);
        let is_falling = gravity.is_some() && !grounded && vel_y < -f32::EPSILON;
        let is_moving = (controlled_movement.is_some() && vel_x_abs > f32::EPSILON)
            || auto_movement
                .map(|a| a.enabled && a.direction.length_squared() > f32::EPSILON)
                .unwrap_or(false)
            || moving_platform.map(|mp| mp.can_move()).unwrap_or(false);

        // --- Apply priority ladder (highest-priority condition wins) ---
        let new_state = if is_range_attacking {
            EntityState::RangeAttacking
        } else if is_damaged {
            EntityState::Damaged
        } else if is_jumping {
            EntityState::Jumping
        } else if is_crouching {
            EntityState::Crouching
        } else if is_falling {
            EntityState::Falling
        } else if is_moving {
            EntityState::Moving
        } else {
            EntityState::Idle
        };

        // --- Check lock_ms before allowing the transition ---
        if new_state != sm.state {
            let locked = check_lock_ms(&sm, &entity_type_assets, spawned);
            if !locked {
                apply_transition(&mut sm, new_state, &entity_type_assets, spawned, &mut collider, &mut anim_cfg, &mut sprite);
            }
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Returns `true` if the current state's `lock_ms` has not elapsed yet.
fn check_lock_ms(
    sm: &StateMachine,
    eta: &Option<Res<EntityTypeAssets>>,
    spawned: Option<&SpawnedLevelEntity>,
) -> bool {
    let (Some(eta), Some(sel)) = (eta.as_deref(), spawned) else {
        return false;
    };
    let state_name = sm.state.to_state_name();
    if let Some(sa) = eta.get_state(&sel.entity_type, state_name) {
        let elapsed_ms = sm.state_time * 1000.0;
        return elapsed_ms < sa.lock_ms as f32;
    }
    false
}

/// Transition to `new_state` and update `AnimationConfig` + `Collider` from the cache.
fn apply_transition(
    sm: &mut StateMachine,
    new_state: EntityState,
    eta: &Option<Res<EntityTypeAssets>>,
    spawned: Option<&SpawnedLevelEntity>,
    collider: &mut Option<Mut<Collider>>,
    anim_cfg: &mut Option<Mut<AnimationConfig>>,
    sprite: &mut Option<Mut<Sprite>>,
) {
    sm.set_state(new_state);

    let (Some(eta), Some(sel)) = (eta.as_deref(), spawned) else {
        return;
    };

    let state_name = new_state.to_state_name();
    let Some(sa) = eta.get_state(&sel.entity_type, state_name) else {
        return;
    };

    // Update AnimationConfig with new state's frames & timer.
    if let Some(ac) = anim_cfg.as_mut() {
        **ac = AnimationConfig::new(sa.frames.clone(), sa.animation_frame_ms);
        // Immediately show the first frame.
        if let Some(spr) = sprite.as_mut() {
            if let Some(h) = sa.frames.first() {
                spr.image = h.clone();
            }
        }
    }

    // Update Collider hitbox.
    if let Some(col) = collider.as_mut() {
        let et = eta.map.get(&sel.entity_type);
        let (sw, sh) = et.map(|e| (e.sprite_width, e.sprite_height)).unwrap_or((128.0, 128.0));
        **col = build_collider_from_box(sa.collider_box.as_deref(), sw, sh);
    }
}
