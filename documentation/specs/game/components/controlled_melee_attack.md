ControlledMeleeAttack

Description
Configuration for player-controlled melee weapons. Stores damage, hit range and a repeating cooldown Timer used by input systems to gate swings.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| damage | integer | 1 | Damage applied per swing. |
| range | number | 12.0 | Melee radius in world units. |
| cooldown | duration/number | 0.3s (repeating) | Time between swings; stored as a repeating Timer. |

Notes
- The cooldown is pre-elapsed so the first melee input is available immediately.

State interactions
- When the player performs a melee swing, systems will apply damage and may cause the `StateMachine` to enter the `MeleeAttacking` state (the component itself has no state field; systems set the state transition signal). 
