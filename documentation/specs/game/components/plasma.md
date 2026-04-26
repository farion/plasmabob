PlasmaBeam

Description
Runtime component representing an active plasma beam effect. It stores the beam origin, direction, current length, an optional lifetime timer and an optional pointer to a projectile entity.

Configuration (JSON keys, types, defaults and explanation)

This component is primarily created at runtime; if authored, the keys are:

| Key | Type | Default | Description |
|---|---:|---:|---|
| origin | [x,y] array | required when authored | Beam origin in world coordinates. |
| direction | number | required when authored | Beam direction (angle). |
| current_length | number | 0.0 | Starting length of the beam. |
| target_projectile | entity id or null | null | Optional projectile entity to track. |
| lifetime | duration | none | Optional Timer to auto-expire the beam. |

Notes
- Typically spawned and managed by runtime systems rather than authored as static component data.
