Team

Description
Simple marker component that assigns a string team name to an entity. Team names are used for friendly-fire rules and aggro-sharing between entities.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| name | string | "Neutral" | Team identifier string. Projectiles and AI use this to avoid friendly fire or share aggro. |

Notes
- Team name is required when present in JSON (macro enforces pick_string_required), but many entities will rely on the default "Neutral" when the component is omitted in authored data.
