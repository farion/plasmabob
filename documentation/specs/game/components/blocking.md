Blocking

Description
Marks an entity as blocking for movement/collision queries. Optionally blocks line-of-sight checks used by AI and vision systems.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| blocks_line_of_sight | boolean | false | If true the collider also obstructs line-of-sight queries (used by AI vision and projectiles). |

Notes
- The component itself is a simple marker with one configurable flag. Collision filtering is handled by Collider/RigidBody and game systems.
