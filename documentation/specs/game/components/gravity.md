Gravity

Description
Component applied to entities affected by gravity. It contains a per-entity scale and grounded flag, plus an extra_accel vector for additional accelerations.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| scale | number | 1.0 | Gravity scale multiplier applied to the global gravity. |
| grounded | boolean | false | Initial grounded state; usually computed by physics at runtime. |
| extra_accel | [x,y] array | [0.0, 0.0] | Additional acceleration applied to the entity (world units/sec^2). |

Notes
- The grounded flag is updated by collision systems; authors normally omit this field.
