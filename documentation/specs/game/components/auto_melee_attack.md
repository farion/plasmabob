AutoMeleeAttack

Description
Autonomous melee attack component for enemies (swipe/bite). Stores damage, attack radius, a repeating cooldown timer and an enabled flag.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| damage | integer | 1 | Damage applied per attack. |
| range | number | 12.0 | Attack radius in world units. |
| cooldown | duration/number | 0.5s (repeating) | Time between swings; stored as a repeating Timer. |
| enabled | boolean | true | Whether the AI melee attack is enabled. |

Notes
- The timer is pre-elapsed so an initial overlap can cause damage immediately.

State interactions
- When the component deals damage or triggers an attack, systems set `just_attacked` for one frame. The state machine uses this to transition the entity into the `MeleeAttacking` state.
