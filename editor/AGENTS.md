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

## Egui layout gotchas (editor-specific)

- ScrollArea content width is remembered across frames: egui will use the previous
  content width as a minimum for the next frame. This can create a feedback loop
  where widgets inside the scroll area request a width that prevents the surrounding
  panel from being resized smaller.

- Recommended pattern to avoid layout feedback loops used in the Entity Type editor:
  1. Read the panel's available width once at the start of the sidebar (before
	 creating the `ScrollArea`). Use that value to compute all column widths for
	 the tables inside the panel.
  2. Use explicit, pre-computed column widths with `Column::exact(width)` instead
	 of `Column::remainder()` or other dynamic sizing variants.
  3. Give `TextEdit` widgets the exact column width via `.desired_width(text_col_w)`
	 so they don't request additional space asynchronously.
  4. Avoid `desired_width(0.0)` (too small) or `desired_width(f32::INFINITY)` (can
	 feed back into the ScrollArea). Using the computed `text_col_w` is stable.

- Optionally, call `ui.set_max_width(ui.available_width())` inside the ScrollArea
  to enforce an upper bound on its content width for the current frame. However,
  the primary fix is to compute and pass down explicit widths before the
  ScrollArea is created.

These rules prevent the sidebar from jumping back to a previous size and ensure
that text fields expand and contract exactly with the panel while the user is
dragging it.

## Internationalization (i18n)

There is no i18n support in the editor. All text is in english and this must stay that way.
