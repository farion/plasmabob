ControlledMovement

Description
Movement tuning for player-controlled entities: walking speed, jump force, double-jump and dash. Includes a runtime jumps_performed counter and optional facing override.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| speed | number | 120.0 | Horizontal movement speed in world units/sec. |
| jump_force | number | 260.0 | Jump impulse strength. |
| allow_double_jump | boolean | true | Allow double jump. |
| jumps_performed | integer | 0 | Runtime counter of jumps performed; rarely authored. |
| dash_force | number | 300.0 | Dash impulse strength. |
| max_speed | number | 0.0 | Optional horizontal speed clamp (0 = no clamp). |
| facing | number | 1.0 | Facing scalar: -1 = left, 1 = right (override). |

Notes
- Most fields are tuning parameters; jumps_performed is managed at runtime by movement systems.
