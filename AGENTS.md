# AGENTS.md — PlasmaBob Codebase Guide

## Stack
- **Rust** (2024 edition)
- **Bevy 0.18.1** (ECS game engine)
- **avian2d 0.6.1** for physics (rigid bodies, colliders, shape casters)
- **bevy_framepace 0.21** for frame-rate limiting
- **serde 1.0.228 / serde_json 1.0.149** for JSON level loading

All mentioned dependencies are compatible with bevy 0.18.1.

## Build & Run
```bash
cargo run             # debug binary with hot reload
cargo build --release # optimised binary
```
Dev profile already sets `opt-level = 1` and `lto = "thin"` for fast iteration.

## Architecture Overview

```
main.rs            ← Entry point with the main menu
src/views/         ← All the UI views
src/game/          ← Everything related to the game itself running a level 
  game_view.rs     ← GameViewPlugin: declares all systems and their ordering
  components/      ← One file per ECS component type; each exports an insert() fn
  systems/         ← One file per ECS system
  systems/common/  ← Reusable helper methods for systems
```

## JSON Assets
- All game data is loaded from JSON files in the `assets/` directory
- This includes world definitions, level layouts, entity types, and story text

### Welt-JSON Schema
`assets/worlds/*.json`



## Level Format 
`assets/worlds/{worldname}/{levelname}_level{number}.json`

## Entity Types Format
`assets/entity_types/*.json`

Entities in PlasmaBob are not hardcoded in Rust but defined via JSON data. Each entity type has a name, a list of
gameplay components, and a map of animations for different states.

## States

States in PlasmaBob are defined via the `EntityState` enum. Each entity can be in one of these states, which affects
its animation, hitbox and gameplay behavior. For example, an entity in the `Walk` state will use the walking animation
and have a different hitbox than when it is in the `Jump` state.

Available states:
- `Default`
- `Walk`
- `Jump`
- `Fight`
- `Hit`
- `Die`

An entity must at least have a `Default` state defined in its `animations` map, but it can have any combination of
the other states as well.

## Animation

## Hitbox Polygons

## Best Practices

### Adding a new game system

Adding a new game system in PlasmaBob is straightforward. You can create a new Rust file under `src/game/systems/` and
define your system there. Then you need to add it to the `GameViewPlugin` in `src/game/game_view.rs` and specify its 
ordering with respect to the other systems. Only one system per file is recommended for better readability and
maintainability.