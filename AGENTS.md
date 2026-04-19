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
logic (components + systems), and data (JSON assets).


Top-level layout (what each area is responsible for):

- `main.rs` — Application bootstrap and global app configuration:
  - Initializes plugins and resources (fonts, i18n, key bindings, audio settings).
  - Registers the main `AppState` and starts the UI via the `views::ViewsPlugin`.
  - Spawns the camera and the main menu UI.

- `src/views/` — All UI views for menus and screens (Start, Settings, About, Story, World map, Load/Win/Lose):
  - Each view lives in its own module and exports a Bevy `Plugin`.
  - `views::ViewsPlugin` composes the individual view plugins and ensures they are added to the app.
  - Views may depend on small helper modules under `src/helper` (i18n, fonts, particles).

- `src/game/` — See `src/games/AGENTS.md`
- `src/game/systems` - See `src/game/systems/AGENTS.md`

- `src/helper/` — Cross-cutting utilities used by both views and game systems (i18n, fonts, audio settings,
  key binding persistence, small particle helpers). Keep pure helpers here to avoid circular dependencies.
d
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


Cross-cutting helpers and global modules live at the crate root and are important for agents to know about:
- `src/i18n.rs` — loads localized strings from `assets/i18n/*.json` and provides `i18n::LocalizedText` usage throughout UI code.
- `src/key_bindings.rs` — persists and loads key bindings; see `KeyBindings::load_from_disk()` usage in `src/main.rs`.
- `src/fonts.rs` — replaces Bevy's default fonts and registers the project's SpaceMono family via a `FontsPlugin`.
- `src/audio_settings.rs` — audio settings persistence used when spawning music (see `MenuMusicEntity` in `src/main.rs`).

## Coding guidelines

- One file per system
- One file per component
- No logic code in mod.rs, only `pub mod` declarations and re-exports if needed.
- No backwards compatibility on changes is required if not explicitly mentioned in the prompt.
- If JSON is parsed, always use completely typed structs with `serde` (no `serde_json::Value` or untyped parsing).

## Internationalization (i18n)

Texts in the game must be localized. The `i18n` module loads localized strings from JSON files in `assets/i18n/`. The
keys translated in those files must be used to bring texts in the game.

## Commenting

- All commenting must happen in english.
- Do not add comments for removed features or code that is not used anymore. If you remove a feature, remove all comments related to it as well.

## Best Practices

## Implementation hints for Bevy 0.18.1

- `Timer::finished()` does not exist anymore, use `Timer::just_finished()` or `Timer::is_finished()` instead.
- `Entity::despawn_recursive()` is now `Entity::despawn()`
- `ui::id_source()` is deprecated, use `ui::id_salt()` instead.