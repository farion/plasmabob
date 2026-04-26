Orientation

Description
Tracks an entity's facing (left/right) and optional surface alignment vector used by rendering and movement systems.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| facing | string | "Right" | Facing direction: "Left" or "Right" (case-insensitive). |
| surface_alignment | [x,y] array | [0.0, 0.0] | Surface alignment vector (defaults to Vec2::ZERO). |

Notes
- Facing is parsed into the FacingDirection enum. Keep surface_alignment at [0,0] for default world-up alignment.

Enums
- FacingDirection options: Left, Right (JSON strings: "left", "right").
