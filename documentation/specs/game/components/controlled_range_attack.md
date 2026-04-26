ControlledRangeAttack

Description
Configuration for player-controlled ranged attacks. Stores damage, range, projectile speed and a one-shot cooldown Timer that is ready on spawn. Contains optional projectile_type and VFX keys.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| damage | integer | 1 | Damage per shot. |
| range | number | 200.0 | Effective range in world units. |
| speed | number | 1200.0 | Projectile speed (world units/sec). |
| cooldown | duration/number | 0.25s (one-shot) | Time between shots; one-shot Timer starts ready so the first shot fires immediately. |
| projectile_type | string or null | null | Optional projectile identifier used by spawn systems. |
| shoot_effect | string or null | "plasma_shoot" | VFX name played on fire. |
| impact_effect | string or null | "plasma_impact" | VFX name for impact. |

Notes
- The cooldown uses TimerMode::Once and is initialised so the weapon is immediately usable on spawn.

State interactions
- When the player fires, systems set `just_fired` for one frame. The state machine reads this signal and may transition the entity to the `RangeAttacking` state.
