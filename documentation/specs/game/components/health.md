Health

Description
Represents an entity's hit points, current and maximum health, and optional despawn-on-death behaviour.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| max | integer | 1 | Maximum HP. |
| current | integer | defaults to max | Starting current HP; if omitted it defaults to `max`. |
| despawn_on_death | boolean | false | If true the entity will be removed after death. |
| despawn_delay_ms | integer | 0 | Delay in milliseconds before fade-out/removal after death. |

Notes
- The JSON override macro ensures current defaults to max if not provided.
