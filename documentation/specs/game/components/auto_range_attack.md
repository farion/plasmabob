AutoRangeAttack

Description
Autonomous ranged attack component for enemies and turrets. Controls damage, range, projectile speed and firing cadence. The component exposes effect keys used by the VFX/spawn systems. The component contains a Timer for cooldown which can be configured from JSON as a duration or interval.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| damage | integer | 1 | Damage dealt per projectile. |
| range | number | 200.0 | Effective range (world units) of the attack. |
| speed | number | 400.0 | Projectile speed (world units/sec). |
| aggro_range | number | 300.0 | Detection radius used to pick a target. |
| cooldown | duration/number | 1.0s (repeating) | Interval between shots. Stored as a repeating Timer. Provide seconds or a timer object in JSON if supported. |
| particle_effect | string or null | "fire" | Optional particle effect key to use for the projectile trail. |
| shoot_effect | string or null | "fire_shoot" | Name of the VFX to play when shooting. |
| impact_effect | string or null | "fire_impact" | Name of the VFX to play on impact. |
| enabled | boolean | true | Whether the attack system should be active for this entity. |

Notes
- The cooldown timer is pre-elapsed on creation so entities can fire immediately when a target appears within aggro_range.
- The macro-based override maps JSON fields into the component using helper parsers (pick_timer supports timer semantics).

State interactions
- When the component fires, systems set `just_fired` for one frame. The state machine uses this signal to transition the entity into the `RangeAttacking` state.
