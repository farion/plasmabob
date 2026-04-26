RigidBody

Description
Stores simple dynamic physics state for an entity: linear velocity, mass and damping/restitution coefficients. A mass of 0.0 or less marks the body as static/immovable.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| velocity | [x,y] array | [0.0, 0.0] | Initial linear velocity (Vec2). |
| mass | number | 1.0 | Mass value. 0.0 or negative marks the body as static. |
| linear_damp | number | 0.0 | Linear drag applied per second. |
| restitution | number | 0.0 | Collision restitution (bounciness). |

Notes
- Use mass=0 for static objects (floors, walls) that should not be moved by physics. The component exposes an is_static() helper at runtime.
