# AGENTS.md — PlasmaBob Codebase Guide

## Stack
 - **Rust** (2024 edition)
 - **Bevy 0.18.1** (ECS game engine)
 - **avian2d 0.6** for physics (rigid bodies, colliders, shape casters)
 - **bevy_framepace 0.21** for frame-rate limiting
 - **serde 1.0 / serde_json 1.0** for JSON level loading

All mentioned dependencies are compatible with bevy 0.18.1.

Note: the authoritative, up-to-date versions are declared in `Cargo.toml` (use it as the source of truth when in doubt).

## Build & Run
```bash
cargo run             # debug binary with hot reload
cargo build --release # optimised binary
```
Dev profile already sets `opt-level = 1` for faster incremental builds and disables LTO to keep build iterations quick (see `Cargo.toml`).

You can pass a level asset path as a CLI argument. The executable normalizes paths so you may provide either a `levels/` path or a `worlds/...` path. Examples:

```bash
cargo run -- levels/level1.json
cargo run -- worlds/auralis/aqueon_level1.json
```

## Architecture Overview

This repository is organised around Bevy's ECS and plugin model. The codebase separates UI (menu/views), game
logic (components + systems), and data (JSON assets). The high-level design emphasizes:

- Small, single‑responsibility systems: one system per file under `src/game/systems/` for clarity and easy ordering.
- Data‑driven entity definitions: entity types and worlds live in `assets/` and are parsed at runtime by `src/level.rs`.
- Explicit module boundaries: `game::systems::systems_api` holds shared components, resources and constants used
  across systems to keep public surface minimal.

Top-level layout (what each area is responsible for):

- `main.rs` — Application bootstrap and global app configuration:
  - Initializes plugins and resources (fonts, i18n, key bindings, audio settings).
  - Registers the main `AppState` and starts the UI via the `views::ViewsPlugin`.
  - Spawns the camera and the main menu UI.

- `src/views/` — All UI views for menus and screens (Start, Settings, About, Story, World map, Load/Win/Lose):
  - Each view lives in its own module and exports a Bevy `Plugin`.
  - `views::ViewsPlugin` composes the individual view plugins and ensures they are added to the app.
  - Views may depend on small helper modules under `src/helper` (i18n, fonts, particles).

- `src/game/` — Game runtime for playing a level:
  - `game_view.rs` — `GameViewPlugin` and the place where systems are declared and ordered. It defines when
    systems run (OnEnter, Update, PostUpdate, OnExit) and groups chain/run_if semantics.
  - `components/` — One file per gameplay component (player, hostile, health, hitbox, plasma, etc.). Each file
    exposes an `insert()` helper where appropriate and component types used by systems.
  - `systems/` — All gameplay systems, organised into subpackages:
    - `gameplay/` — core game mechanics (movement, collisions, combat, entity state sync).
    - `presentation/` — visual/audio follow‑ups (camera, HUD, particles, audio playback, parallax).
    - `maintenance/` — editor/debug helpers and housekeeping (cleanup, debug overlays, pause menu).
    - `setup_spawn/` — initial spawn/setup logic for a level (spawn background, HUD, entities, boundaries).
    - `systems_api.rs` — small shared API of components/resources/constants used across submodules (e.g.
      `GameViewEntity`, `ActiveLevelBounds`, `QuoteCooldown`).

- `src/helper/` — Cross-cutting utilities used by both views and game systems (i18n, fonts, audio settings,
  key binding persistence, small particle helpers). Keep pure helpers here to avoid circular dependencies.

- `src/level.rs` and `assets/` — Level loading, entity type parsing, and JSON schemas:
  - `level.rs` contains the loader that reads JSON from `assets/` and converts it into in-memory `LevelDefinition`.
  - `assets/entity_types/*.json` define entity components, animations and sizes. The runtime uses these to spawn
    entities via `game::components::spawn_entity`.

Design patterns and conventions
- One system per file. Register systems in `GameViewPlugin` and prefer small chains (.chain(), .before(), .run_if()).
- Systems and components access only the minimal `systems_api` surface when they must share state across directories.
- Resources that represent shared runtime state are declared with `#[derive(Resource)]` and initialised in `GameViewPlugin`
  or `main.rs` as appropriate.
- UI stacking / z-order: Bevy UI `ZIndex` is used where needed (see `main.rs` main menu example).

## Error and logging policy
Goal: consistent, typed errors and structured logging to improve diagnostics and make APIs more robust.

- Use `thiserror` for module/domain-specific error types instead of raw `String` errors. Example:
  - `#[derive(thiserror::Error, Debug)] pub enum LoadLevelError { Io(#[from] std::io::Error), Parse(#[from] serde_json::Error), ... }
- Use `tracing` (or `log`) for structured logs. Initialize e.g. `tracing_subscriber::fmt::init()` in `main.rs`.
- Resources should not store raw `String` errors. Instead:
  - store typed errors (e.g. `Option<LoadLevelError>`), or
  - store a UI-friendly formatted message (`Option<String>`) that is derived from the typed error at the UI boundary.
- Error propagation: use `?` with `From` conversions (`#[from]`) on error enums to reduce boilerplate.
- Localized user messages: derive localized, UI-friendly text from typed errors at the UI boundary (do not embed localization inside domain error types).

Practical migration guidance:
- First introduce typed error enums for the main domains (Level/World/Story/Entity) using `thiserror`.
- Replace `Result<..., String>` with `Result<..., TypedError>` incrementally; convert to strings only at UI boundaries as needed.
- Centralize Asset I/O errors in `src/helper/asset_io.rs` and provide `From` conversions so `?` works naturally everywhere.

Benefits:
- Better debugging (structured logs, backtraces, machine-readable errors)
- Fewer silent failures or error-mangling from unstructured strings
- Simple conversion to localized UI text in one place


## Testing and editor
- Unit tests for pure functions live alongside the module (`#[cfg(test)]` blocks). Integration smoke tests are
  exercised by running the game and using `cargo run` with a level path.
- There is a separate `editor/` binary that operates on the same JSON assets; the editor code is kept in the
  `editor/` folder to avoid coupling the runtime.


### Module Dependency Graph

src -> src::game -> src::helper -> src::views
src::views -> src::game
src::game -> src::game::components src::game -> src::game::systems
src::game::systems -> src::game::components src::game::systems -> src::helper (durch Teilmodule wie presentation/gameplay)
src::game::systems::gameplay -> src::game::components src::game::systems::gameplay -> src::helper
src::game::systems::presentation -> src::game::components src::game::systems::presentation -> src::helper
src::game::systems::maintenance -> src::game::components
src::game::systems::setup_spawn -> src::game::components src::game::systems::setup_spawn -> src::helper

Note: there is also a separate editor executable in `editor/` (see `editor/AGENTS.md`). The editor is built with Bevy and `bevy_egui` and operates on the same JSON assets in `assets/` but does not run game logic.

Cross-cutting helpers and global modules live at the crate root and are important for agents to know about:
- `src/i18n.rs` — loads localized strings from `assets/i18n/*.json` and provides `i18n::LocalizedText` usage throughout UI code.
- `src/key_bindings.rs` — persists and loads key bindings; see `KeyBindings::load_from_disk()` usage in `src/main.rs`.
- `src/fonts.rs` — replaces Bevy's default fonts and registers the project's SpaceMono family via a `FontsPlugin`.
- `src/audio_settings.rs` — audio settings persistence used when spawning music (see `MenuMusicEntity` in `src/main.rs`).

## JSON Assets
- All game data is loaded from JSON files in the `assets/` directory
- This includes world definitions, level layouts, entity types, and story text

### Welt-JSON Schema
`assets/worlds/*.json`

## Level Format 
`assets/worlds/{worldname}/{levelname}_level{number}.json`
Example: `assets/worlds/auralis/aqueon_level1.json` (see `assets/worlds/auralis/`).

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

## Internationalization (i18n)

Texts in the game must be localized. The `i18n` module loads localized strings from JSON files in `assets/i18n/`. The
keys translated in those files must be used to bring texts in the game.

## Commenting

All commenting must happen in english.

## Best Practices

### How to add a new gameplay system
1. Create a file under the appropriate submodule, e.g. `src/game/systems/gameplay/my_new_system.rs`.
2. Add `pub mod my_new_system;` to `src/game/systems/gameplay/mod.rs`.
3. Register the system in `src/game/game_view.rs` in the correct scheduling group (OnEnter/Update/PostUpdate) and
   use `.before()` / `.after()` / `.chain()` to position it relative to existing systems.
4. If the system needs to share data with others, add a small type to `systems_api.rs` or a new `Resource` in
   `game_view.rs` as needed.
