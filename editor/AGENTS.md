# AGENTS.md — Editor Module Refactor Rules

Dieses Dokument beschreibt verbindliche Regeln und einen Arbeitsplan für zukünftige Agents,
die Refactorings oder Umstrukturierungen des `editor`-Crates durchführen sollen.

Ziel
- Aufteilung des Editor-Codes in sinnvolle Module/Ordner, so dass:
  - Keine Quelldatei mehr als 300 Zeilen enthält.
  - Dashboard-relevanter Code unter `editor/src/dashboard` liegt (crate::dashboard).
  - Level-Editor relevanter Code unter `editor/src/level` liegt (crate::level).
  - Entity-Type-Editor relevanter Code unter `editor/src/entity_types` liegt (crate::entity_types).
  - Cross-cutting concerns (geteilte Ressourcen, Utilities, IO, Sync, Table-UI) unter `editor/src/core` liegen (crate::core).
  - Model / DTO Code bleibt unter `editor/src/model.rs` (crate::model).

Grundregeln
- Bevor Änderungen vorgenommen werden: Einen klaren Plan hier in dieser Datei hinterlegen und vom Benutzer bestätigen lassen.
- Änderungen erst nach ausdrücklicher „Mach los“-Bestätigung durchführen.
- Keine Commits oder Pushes ohne explizite Erlaubnis des Benutzers. Änderungen können lokal angewendet werden, aber committet wird nur auf Anforderung.
- Dateien niemals größer als 300 Zeilen halten; bei Bedarf weiter aufteilen.
- Minimale Änderungen bevorzugen: nur modulare Aufteilung und Pfad-Anpassungen, keine logischen API-Änderungen außer wenn unumgänglich.
- Include-/Build-generierte Dateien (z. B. `include!(concat!(env!("OUT_DIR"), "/component_attr_map.rs"))`) unverändert belassen — nicht verschieben.
- Shared Ressourcen (z. B. `EditorDocument`, `ActiveCharacter`, `ComponentValueMapping`, `ToastState`, `ColumnWidths`) kommen in `crate::core`.
- Level-Editor ist ein eigenständiges Modul `crate::level` (unter `editor/src/level`).

Empfohlene Endstruktur
- editor/src/
  - main.rs
  - model.rs
  - core/           # nur cross-cutting (crate::core)
    - mod.rs
    - state.rs
    - io.rs
    - sprites.rs
    - sync.rs
    - table_ui.rs
    - utils.rs
  - level/          # Level-Editor (crate::level)
    - mod.rs
    - ui.rs
    - scene.rs
    - input.rs
    - state.rs
  - entity_types/   # Entity-Type-Editor (crate::entity_types)
    - mod.rs         # enthält auch include! für component_attr_map.rs
    - hitbox.rs
    - array_editor.rs
    - components_sidebar.rs
    - preview.rs
  - dashboard/      # Dashboard (crate::dashboard)
    - mod.rs

Vorgehensweise beim Refactor (sichere, iterative Schritte)
1. Planerstellung: Änderungen hier dokumentieren und Bestätigung vom Benutzer einholen.
2. Stubs/Module anlegen: Verzeichnisse und minimale `mod.rs`/Stub-Funktionen erstellen, damit die Modul-Hierarchie existiert.
3. Shared-Types nach `core/state.rs` extrahieren (EditorDocument, ToastState, ActiveCharacter, ComponentValueMapping, ColumnWidths, EditorMode etc.).
4. IO- und Sync-Funktionen in `core/io.rs`, `core/sprites.rs`, `core/sync.rs` aufteilen.
5. Entity-Type-Editor in `editor/src/entity_types` aufteilen; `include!(concat!(env!("OUT_DIR"), "/component_attr_map.rs"));` in `entity_types/mod.rs` belassen.
6. Level-Editor spezifische Dateien in `editor/src/level` verschieben.
7. Dashboard komplett in `editor/src/dashboard/mod.rs` implementieren (alte top-level dashboard.rs ersetzen).
8. Iterativ `cargo check` im `editor`-Ordner ausführen und alle Compiler-Fehler beheben (modular, nach jedem größeren Schritt prüfen).
9. Nach erfolgreichem `cargo check` endgültig alte Dateien entfernen und abschließende Aufräumarbeiten durchführen.

Praktische Prüfungen
- Verwende `cargo check` im `editor`-Ordner zur Validierung nach jeder größeren Änderung.
- Suche/Ersetze `crate::editor::` Referenzen durch die neuen Modulpfade (z. B. `crate::core::`, `crate::level::`, `crate::entity_types::`, `crate::dashboard::`).

Typische Fehler & Gegenmaßnahmen
- "unresolved import" → fehlende `mod`-Deklaration: `mod.rs` oder `pub(crate) use` ergänzen.
- Sichtbarkeitsfehler (`private type`) → Sichtbarkeit vorsichtig auf `pub(crate)` erweitern.
- Zirkuläre Abhängigkeiten → extrahiere Traits/Abstraktionen nach `core` oder verschiebe Implementierungen zur Vermeidung von cycles.
- include!-Fehler bei build-gen → `include!` niemals verschieben; stelle sicher, dass build.rs das generierte Artefakt produziert.

Commit-Policy
- Standard: keine automatischen Commits. Commits und Branches nur auf ausdrückliche Anweisung des Benutzers.

Tests
- Unit-Tests bleiben in den jeweiligen Modulen (`#[cfg(test)]`). Große io/sync-Tests sollten nach `core/sync.rs` verschoben werden, wenn sie zu diesem Modul gehören.

Beispiel-Checkliste (vor Beginn der Codemodifikation)
1. Benutzer hat die Änderungen hier bestätigt.
2. Backup/Branching-Entscheidung geklärt (soll committet werden oder nicht?).
3. Stub-Module erstellt und `cargo check` läuft.
4. Iterative Migration mit `cargo check` nach jedem Schritt.

Diese Regeln sind verbindlich für Agents, die an der Editor-Umstrukturierung arbeiten. Änderungen an dieser Datei sollten nur nach Absprache mit dem Repository-Besitzer erfolgen.
