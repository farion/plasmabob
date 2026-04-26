StateMachine

Description
Represents an entity's current high-level state (Idle, Moving, MeleeAttacking, RangeAttacking, Damaged, Dying, Dead, Jumping, Falling, Crouching, Collected) and holds per-state authored configuration used by animation, sound and hitbox selection.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| initial_state | string | "idle" | Initial state name (see valid names in Notes). |
| dying_duration_secs | number | 1.0 | Time in seconds an entity remains in Dying before becoming Dead. |
| states | object | {} | Map from state-name -> StateConfig. StateConfig contains fields such as animation, animation_frame_ms, collider_box, lock_ms, sound_start/loop/end and other per-state options. See level config schema for StateConfig for exact keys. |

Notes
- Valid state names: idle, moving, jumping, falling, damaged, dying, collected, dead, melee_attacking, range_attacking, crouching.
- The component runtime includes derived fields (prev_state, state_time and a typed HashMap of states); only the authored fields appear in JSON.

EntityState enum values
- Idle
- Moving
- Jumping
- Falling
- Damaged
- Dying
- Collected
- Dead
- MeleeAttacking
- RangeAttacking
- Crouching

State interactions (components that trigger or depend on these states)
- AutoMeleeAttack / ControlledMeleeAttack: set signals that should transition the entity to `MeleeAttacking` when an attack fires (`just_attacked` / attack event). 
- AutoRangeAttack / ControlledRangeAttack: set signals that transition to `RangeAttacking` (`just_fired`).
- Damageable: sets `Damaged` state for `damaged_duration_secs` after being hit.
- Health: when `current` <= 0 the state machine should move to `Dying` and then `Dead` after `dying_duration_secs`.
