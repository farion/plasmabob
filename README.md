# PlasmaBob

PlasmaBob is a 2D platformer game where players control a hero named Bob to rescue the world.

This was started on a 24h hackathon at exxeta to explore the Bevy game engine and to have fun. The game is still in early development, but the basic level loading and entity system is in place.

Have Fun!

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

