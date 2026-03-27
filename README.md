# PlasmaBob

Minimal Bevy prototype for a 2D platformer.

## Run

```bash
cargo run
cargo run -- level1.json
cargo run -- levels/level1.json
```

## Current level format

- Level files live under `assets/levels/`
- Coordinates use a bottom-left origin: `(0, 0)` is the lower-left corner of the screen
- `entity_types` define size, animations, and gameplay components
- `entity_types` also define `hitbox` polygons (local points, origin at entity bottom-left)
- `entities` place concrete instances into the level

## Gameplay components

Each gameplay component has its own Rust file under `src/components/`:

- `floor`
- `npc`
- `hostile`
- `player`

