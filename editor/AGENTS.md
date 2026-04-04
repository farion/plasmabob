# AGENTS.md — PlasmaBob Editor Guide

## Editor Overview

The editor helps developers creating worlds and levels and entity types. It is a separate executable
that can be found in the `editor/` directory. It is built using Bevy but it is not a plugin and
does not run the game logic. It basically just works with the json files in the `assets/` directory
and provides a visual interface to edit them.

## Build & Run

Build
```bash
cargo build
```

Run
```bash
cargo run
```

## Editor Features

### Dashboard

The dashboard shows three columns for the three main data types: `Worlds`, `Levels`, and `Entity Types`.
`Worlds` shows the worlds that are defined in `assets/worlds/`, `Levels` shows the levels that are defined in 
`assets/levels/`, and `Entity Types` shows the entity types that are defined in `assets/entity_types/`.
`Worlds` is basically only a filter for the levels. When you click on a world, only the levels that belong to that
world are shown in the `Levels` column. When you click on a level, the level editor is opened. When you click on an
entity type, the entity type editor is opened.

### Level Editor

The level editor shows a visual representation of the level. You can see the entities that are placed in the level
and you can move them around. You can also add new entities by clicking on the `Add Entity` button. When you click 
on an entity, you can edit its properties in the right sidebar.

### Entity Type Editor

The entity type editor shows a visual representation of the entity type. You can see the animation sprites and the
hitbox for each state.

## Technical Details

- Rust
- Bevy
- In contrast to the game the editor is using bevy_egui for the UI

## Internationalization (i18n)

There is no i18n support in the editor. All text is in english and this must stay that way.
