Damageable

Description
Marks an entity as able to take damage and contains a short damaged-state duration used to trigger hit reactions and temporary invulnerability.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| damaged_duration_secs | number | 0.5 | Seconds the entity stays in the Damaged state after being hit. |

Notes
- damaged_timer is a runtime field set when the entity takes damage and counts down; it is not authored.
