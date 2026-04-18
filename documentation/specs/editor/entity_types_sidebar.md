# Entity Types Sidebar — Spec

Goal
----
Describe the new right‑side sidebar UI for the Entity‑Types editor. The sidebar replaces the previous top‑placed components view and shows all components of an entity type together with their attributes and values in a scrollable, categorized table with 2 columns (Attribute | Value).

Main Requirements
-----------------
- Right‑placed, resizable panel (default width 300 px).
- Top: pulldown (ComboBox) containing only the components that are not currently present, plus an explicit "Add" button. Selecting an item does not add it; the Add button must be clicked.
- Main area: long, scrollable table — for each existing component a category (CollapsingHeader). Each category shows two columns: Attribute | configured value.
- If an attribute has no explicit value, display a hint (muted/italic): mapping default or "(not set)". Defaults are shown UI‑only and are not written into JSON automatically.
- Each attribute has an editor widget suitable for its type (number, string, enum, array, waypoint array, complex JSON fallback).
- Each attribute has a small Clear/Reset button with a tooltip. Clear removes the explicit override (the key is removed from the JSON representation); the UI falls back to showing the mapping default.
- Arrays are ordered and provide Up/Down, Remove and Add; waypoints are arrays of [x,y].
- Removing a whole component is done via a Remove button in the category header and opens a centered, modal confirmation dialog.
- Changes are written into the already existing staged data (either `EditorDocument` or `hitbox_editor.edited_entity_types`) and persisted by the existing save logic (Ctrl+S / Save‑and‑Close).
- As few changes as possible to the game code; the editor manipulates JSON/serde internally. Optionally small public helpers can be added in `src/game/level/types.rs` if desired.
- If attributes are edited by the user, the table must not be hidden.
- the table columns must have the same alignment over categories.
- The clear button must only be visible if there is a value set. If the default is shown, the clear button should not be visible.

UI Details
----------
Panel
- Implementation: `egui::SidePanel::right("entity_type_components_sidebar")`
- Properties: `.resizable(true).default_width(300.0).min_width(200.0).max_width(600.0)`
- Inner wrapper: `egui::ScrollArea::vertical()` — the long table is scrollable.

Header (top)
- Label: "Add component:"
- ComboBox: shows `add_options` (result of `scan_game_components()` minus `components_snapshot`).
- Add‑button: explicit; only when clicked the selected component is added.
- Add behavior:
  - `new_components = components_snapshot + chosen`
  - call `set_component_names(&new_components)` on the staged entity type (document or `edited_entity_types`)
  - mark `hitbox_editor.dirty_entity_types.insert(selected_name.clone())`
  - Note: attributes are not written as keys into JSON automatically; UI shows mapping defaults (merged view).

Components list (main area)
- For each component:
  - `egui::CollapsingHeader::new(component_name).default_open(true)`
  - Right in the header: small Remove button (opens modal confirm).
  - Body: two‑column layout (Attribute | Value editor)

Attribute sources and ordering
- Primary: `ComponentValueMapping` (if present) — defines attributes, types, enum options, optional component defaults.
- Fallback: if mapping is missing, use the keys from the existing `components` object structure.
- Sorting: alphabetical or mapping order (mapping first, stable) to keep UI stable.

Editor widgets per type
- number → `egui::DragValue<f64>` (or `f32`) + rudimentary type checking (parseable). Inline visual error for invalid input.
- string → `egui::TextEdit::singleline`
- enum → `egui::ComboBox` with options from mapping; default (when no explicit value) = component default (if present) otherwise first option
- array<string|number> → ordered list UI: per entry an editor (text/number) with Up/Down + Remove; Add item button inserts an empty or sensible default entry
- waypoints → ordered list of `[x,y]` (each entry two `DragValue`s for x,y) + Up/Down + Remove + Add waypoint (default `[0,0]`)
- complex/object → raw JSON TextArea (multiline) as edit fallback

Defaults, display & clear
- Display priority: 1) explicit value (normal text), 2) mapping/component default (muted/italic with tooltip), 3) "(not set)" (muted)
- Tooltip at default hint: "component default" or "first enum option" to indicate the source
- Clear (Reset) button:
  - Tooltip: "Reset to default (removes explicit override from JSON)"
  - Behavior: removes the explicit key from the `components` object (JSON remains minimal). UI then shows mapping default or "(not set)".
- IMPORTANT: Mapping defaults are not written into JSON automatically (UI only). This keeps files compact.

Remove component (delete component entirely)
- Remove button in header opens a centered modal Confirm (`egui::Window::new("Confirm Remove")` anchored CENTER_CENTER).
- Text: "Remove component 'X' from entity type 'Y'?"
- Buttons: "Remove" / "Cancel"
- On Remove: compute `new_components = components_snapshot without component` and call `set_component_names(&new_components)` (staged). Mark dirty.

Dataflow / Persistence
- Write paths (staged):
  - If `EditorDocument` is present: write into `document.entity_types.get_mut(&selected_name)`
  - Else: write into `hitbox_editor.edited_entity_types.get_mut(&selected_name)`
- Save: existing Ctrl+S / Save‑and‑Close logic remains and uses `component_names()` + stored hitboxes; no change needed.
- Editor‑side changes are performed via serde JSON→Map→typed roundtrip or via editor helpers (no changes required in the game internals).

Validation
- Rudimentary type checking while editing (e.g., number input must parse); inline error indication.
- No range checking / clamping.

Arrays & ordering
- Arrays are ordered. Provide Up/Down, Remove and Add.
- Waypoints are `[x,y]` arrays; order is preserved in JSON on save.

Reuse / game code
- Minimal changes to game code: editor manipulates JSON/serde internally. This avoids breaking changes in the game.
- Optionally small public helpers in `src/game/level/types.rs` can be created (e.g., default lookup, merge functions) so game and editor share the same logic. Only add if necessary.

UX details
- Default width 300px; user can resize.
- ComboBox IDs should use `format!("add_component_cb_{}", selected_name)` for stability.
- For long arrays: normal ScrollArea is sufficient; no virtualization needed.

Test cases (manual)
- Add component → component appears in components list (staged) → UI shows attributes with mapping defaults without writing them to JSON
- Edit primitive / enum / array / waypoint → value is staged; `hitbox_editor.dirty_entity_types` is set
- Clear attribute → explicit key removed; UI shows default/"(not set)"
- Remove component → modal confirm → component removed (staged)
- Save (Ctrl+S) → `assets/entity_types/<name>.json` contains only explicit overrides and a minimal components list as before

Open items / confirmations
- Add: component is added to the components list; defaults are UI‑only and not written to JSON — confirmed
- Clear: removes explicit key (does not replace it with explicit default) — confirmed
- Waypoints: format is `[x,y]` — confirmed
- Enum default: component default when present, otherwise first option — confirmed

Next steps
- After finalizing this spec: produce a precise patch checklist (file/section/lines), or optionally direct implementation diffs based on this spec (on request).

---
Date: 2026-04-18
Author: Spec automatically generated (on request)