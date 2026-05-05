Collider

Description
Defines a local collision shape and offset used by the physics and query systems. The runtime Collider currently supports rectangle shapes expressed via half-extents.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| offset | [x,y] array | [0.0, 0.0] | Local offset of the collider relative to the entity transform. |
| shape | object | { "rectangle": { "half_extents": [8.0, 8.0] } } | Shape object. Rectangle form specifies half_extents in world units. |

Notes
- The component previously supported multiple shapes/flags but was simplified to rectangle-only to reduce unused code paths.
