# AGENTS.md — PlasmaBob Codebase Guide

## Stack
- **Rust** (2024 edition) + **Bevy 0.15** (ECS game engine)
- **avian2d 0.2** for physics (rigid bodies, colliders, shape casters)
- **bevy_framepace** for frame-rate limiting
- **serde / serde_json** for JSON level loading

## Build & Run
```powershell
cargo run                          # default level (level1.json)
cargo run -- level1.json           # explicit level (assets/ prefix is stripped automatically)
cargo run -- levels/level1.json    # also accepted
cargo build --release              # optimised binary
```
Dev profile already sets `opt-level = 1` and `lto = "thin"` for fast iteration.

## Architecture Overview

```
main.rs          ← AppState FSM, plugin registration, CLI arg → LevelSelection resource
src/views/       ← One file per non-game screen (start, load, lose, win, settings, about)
src/game/
  game_view.rs   ← GameViewPlugin: declares all systems and their ordering
  level.rs       ← JSON loading (CachedLevelDefinition resource), all level data structs
  components/    ← One file per ECS component type; each exports an insert() fn
  systems/       ← Included via #[path] attributes inside game_view.rs
```

`game_view.rs` uses `#[path = "systems/xxx.rs"] mod xxx;` to pull in system files as
private submodules — **do not add `pub mod` entries for them in `mod.rs`**.

## State Machine
`AppState` enum in `main.rs`: `MainMenu → StartView → LoadView → GameView → LoseView / WinView`  
Each state has `OnEnter`, `Update` (`.run_if(in_state(…))`), and `OnExit` schedules.

## Level Format (`assets/levels/`)
- `assets/entity_types/*.json` — eine Datei pro Entity-Type (components, animations, hitbox, size, health, damage)
- `levelN.json` — terrain, music, quotes, bounds, and entity instances
- **Coordinates**: bottom-left origin `(0, 0)`; `x/y` in `EntityDefinition` are bottom-left of the entity
- `z_index` on instances controls draw order (higher = in front)
- `entity_types_path` defaults to `"entity_types"` (Ordner unter `assets/`)

Adding a new entity type: create `assets/entity_types/<name>.json` with the `component` array and
`animations` map, then place instances in the level JSON.

## Component System
Each component module under `src/game/components/` exports `pub(crate) fn insert(entity: &mut EntityCommands)`.  
`spawn_entity()` in `components/mod.rs` reads the `"component"` array from JSON and dispatches to these functions.

**Known component names** (as used in JSON):
`collision`, `doodad`, `exit`, `floor`, `hostile`, `moving`, `npc`, `player`

Physics assignment rules in `spawn_entity()`:
- `player` or `moving` → `RigidBody::Dynamic` + polygon collider from hitbox
- `collision` only → `RigidBody::Static`
- `collision` + `moving` → also sets `CollisionLayers` (layer mask `0b0010` / `0b1101`)

## Animation
`EntityState` variants (`Default`, `Walk`, `Jump`, `Fight`, `Hit`, `Die`) map to the matching key in the
`animations` map. Changing state is done via `AnimationState::set(next)` — it no-ops if state is unchanged.  
`PreloadedAnimations` caches `Handle<Image>` for all frames at spawn time.

## Hitbox Polygons
- Points are in local space with **bottom-left as origin** (pixel coords from image)
- `centered_hitbox_polygon()` converts them to entity-centred coordinates for avian2d
- `gen_hitbox.py` can be used to extract polygon points from sprite images

## Key Constants (game_view.rs)
| Constant | Value |
|---|---|
| `PLAYER_MOVE_SPEED` | 320.0 px/s |
| `PLAYER_JUMP_SPEED` | 700.0 px/s |
| `MOVING_NPC_MAX_DISTANCE_FROM_ORIGIN` | 500.0 px |
| `PLAYER_INVINCIBILITY_SECONDS` | 1.0 s |
| `SHOW_HITBOX_DEBUG_LINES` (main.rs) | `false` |

## Debug Mode
Press **F1** to toggle hitbox debug lines at runtime (also controlled by `SHOW_HITBOX_DEBUG_LINES`).  
`DebugRenderSettings` resource is inserted in `main.rs` at startup.

## Audio
Only `.ogg` files are supported for music and quotes. `AudioSettings` resource (persisted across states)
holds volume levels. `CombatSoundEffects` resource holds handles for plasma and enemy-death sounds.

## Adding a New Game System
1. Create `src/game/systems/my_system.rs`
2. Add `#[path = "systems/my_system.rs"] mod my_system;` inside `game_view.rs`
3. Register the system in `GameViewPlugin::build` with appropriate ordering constraints

