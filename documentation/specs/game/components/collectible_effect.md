CollectibleEffect

Description
Describes the effect applied when a collectible is picked up (runtime component). Currently supports healing the picker.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| heal | integer | 0 | Amount of HP to restore to the collector on pickup. |

Notes
- The pickup system consumes this component when an entity is collected. Extendable for more effect types in future.
