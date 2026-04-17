use bevy::prelude::*;

use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::auto_melee_attack::AutoMeleeAttack;
use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::{
    AutoMovement, Collider, ControlledMovement, Damageable, EntityState, Gravity, Health,
    MovingPlatform, RigidBody, StateMachine,
};
use crate::game::runtime_components::{AnimationConfig, SpawnedLevelEntity};
use crate::game::setup::collider_helper::build_collider_from_box;
use crate::game::setup::entity_type_assets::EntityTypeAssets;
use crate::helper::key_bindings::KeyBindings;
use crate::game::setup::flip_utils::adjust_new_collider_preserve_world_center;

/// Drives every entity's `StateMachine` state each frame based on movement, physics,
/// combat and damage signals.
///
/// Priority order (highest wins):
///   Dead > Dying > MeleeAttacking > RangeAttacking > Damaged > Jumping / Crouching / Falling > Moving > Idle
///
/// Rules:
/// - **Dead**: terminal — no further transitions.
/// - **Dying**: waits `dying_duration_secs` (tracked via `state_time`) then becomes Dead.
/// - **Dying (new)**: triggered when `Health.is_dead()` is true.
/// - **MeleeAttacking**: one-frame signal from `AutoMeleeAttack.just_attacked`; cleared here.
/// - **RangeAttacking**: one-frame signal from `ControlledRangeAttack.just_fired`; cleared here.
/// - **Damaged**: `Damageable.damaged_timer > 0` (set by collision systems; ticked down here).
/// - **Jumping**: `ControlledMovement` entity, airborne, moving upward.
/// - **Crouching**: `ControlledMovement` entity, crouch key held.
/// - **Falling**: any entity with `Gravity`, airborne, moving downward.
/// - **Moving**: non-zero horizontal velocity (`ControlledMovement` / `AutoMovement`) or active `MovingPlatform`.
/// - **Idle**: default when no other condition applies.
const COLLECTED_DURATION_SECS: f32 = 1.0;
const COLLECTED_UPWARD_SPEED: f32 = 80.0;

pub fn state_machine_update_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    entity_type_assets: Option<Res<EntityTypeAssets>>,
    mut entities: Query<(
        Entity,
        &mut StateMachine,
        Option<&mut Damageable>,
        Option<&Health>,
        Option<&mut ControlledRangeAttack>,
        Option<&mut AutoRangeAttack>,
        Option<&mut AutoMeleeAttack>,
        Option<&ControlledMovement>,
        Option<&AutoMovement>,
        Option<&MovingPlatform>,
        Option<&Gravity>,
        Option<&RigidBody>,
        Option<&mut Collider>,
        Option<(&mut AnimationConfig, &mut Sprite, &mut Transform)>,
        Option<&SpawnedLevelEntity>,
    )>,
) {
    let dt = time.delta_secs();
    let crouch_key = key_bindings.crouch;

    for (
        entity,
        mut sm,
        mut damageable,
        health,
        mut controlled_attack,
        mut auto_range_attack,
        mut auto_melee,
        controlled_movement,
        auto_movement,
        moving_platform,
        gravity,
        rigid_body,
        mut collider,
        mut anim_s_t,
        spawned,
    ) in &mut entities
    {
        // Always advance the state timer — `set_state()` resets it on transitions.
        sm.tick(dt);

        // Tick the damaged timer every frame so it counts down independently of state.
        if let Some(ref mut dmg) = damageable {
            dmg.damaged_timer = (dmg.damaged_timer - dt).max(0.0);
        }

        // --- Collected: play upward + fade animation then despawn after fixed duration ---
        if sm.is(EntityState::Collected) {
            if let Some((_, spr, tr)) = anim_s_t.as_mut() {
                // Move upward and fade alpha over the duration.
                tr.translation.y += COLLECTED_UPWARD_SPEED * dt;
                let frac = (sm.state_time / COLLECTED_DURATION_SECS).clamp(0.0, 1.0);
                // Use srgba with preserved white tint; if a sprite uses a different
                // tint this will be overridden by the state's configured frame.
                spr.color = Color::srgba(1.0, 1.0, 1.0, (1.0 - frac).clamp(0.0, 1.0));
            }
            if sm.state_time >= COLLECTED_DURATION_SECS {
                commands.entity(entity).try_despawn();
            }
            continue;
        }

        // --- Terminal state: Dead does not transition further ---
        if sm.is(EntityState::Dead) {
            continue;
        }

        // --- Dying: check whether the timer has elapsed and transition to Dead ---
        if sm.is(EntityState::Dying) {
            if sm.state_time >= sm.dying_duration_secs {
                apply_transition(&mut sm, EntityState::Dead, &entity_type_assets, spawned, &mut collider, &mut anim_s_t);
            }
            continue;
        }

        // --- New death: health just reached zero ---
        if let Some(hp) = health {
            if hp.is_dead() {
                apply_transition(&mut sm, EntityState::Dying, &entity_type_assets, spawned, &mut collider, &mut anim_s_t);
                continue;
            }
        }

        // --- Read and immediately clear the one-frame range-attack signal ---
        let is_range_attacking = controlled_attack
            .as_ref()
            .map(|a| a.just_fired)
            .unwrap_or(false)
            || auto_range_attack
                .as_ref()
                .map(|a| a.just_fired)
                .unwrap_or(false);
        if let Some(ref mut atk) = controlled_attack {
            atk.just_fired = false;
        }
        if let Some(ref mut atk) = auto_range_attack {
            atk.just_fired = false;
        }

        // --- Read and immediately clear the one-frame melee-attack signal ---
        let is_melee_attacking = auto_melee
            .as_ref()
            .map(|a| a.just_attacked)
            .unwrap_or(false);
        if let Some(ref mut ma) = auto_melee {
            ma.just_attacked = false;
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
        let new_state = if is_melee_attacking {
            EntityState::MeleeAttacking
        } else if is_range_attacking {
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
                apply_transition(&mut sm, new_state, &entity_type_assets, spawned, &mut collider, &mut anim_s_t);
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
    anim_s_t: &mut Option<(Mut<AnimationConfig>, Mut<Sprite>, Mut<Transform>)>,
) {
    sm.set_state(new_state);

    let (Some(eta), Some(sel)) = (eta.as_deref(), spawned) else {
        return;
    };

    let state_name = new_state.to_state_name();
    let Some(sa) = eta.get_state(&sel.entity_type, state_name) else {
        return;
    };

    // Update Collider hitbox first and preserve its world centre so the
    // new animation frame (set below) is drawn with the correct pivot and
    // does not cause a visible flicker.
    if let Some(col) = collider.as_mut() {
        let et = eta.map.get(&sel.entity_type);
        let (sw, sh) = et.map(|e| (e.sprite_width, e.sprite_height)).unwrap_or((128.0, 128.0));

        // Capture old collider offset before replacing so we can preserve
        // the world centre.
        let old_col_x = (**col).offset.x;

        // Build the new collider from the state's config.
        let mut new_col = build_collider_from_box(sa.collider_box.as_deref(), sw, sh);

        // Determine whether the sprite is currently flipped so the new collider
        // can be mirrored accordingly and positioned to keep the world centre.
        let is_flipped = anim_s_t.as_ref().map(|(_, spr, _)| spr.flip_x).unwrap_or(false);
        adjust_new_collider_preserve_world_center(anim_s_t.as_ref().map(|(_, _, tr)| &**tr).unwrap_or(&Transform::default()), old_col_x, &mut new_col, is_flipped);

        **col = new_col;
    }

    // Update AnimationConfig with new state's frames & timer and then set
    // the first frame. Doing this after the collider/transform adjustment
    // prevents the first frame from being shown relative to the old pivot.
    if let Some((ac, spr, _tr)) = anim_s_t.as_mut() {
        **ac = AnimationConfig::new(sa.frames.clone(), sa.animation_frame_ms);
        if let Some(h) = sa.frames.first() {
            spr.image = h.clone();
        }
    }
}
