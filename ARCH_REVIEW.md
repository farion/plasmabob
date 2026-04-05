# Architektur-Review `src/` (PlasmaBob)

## Scope
Diese Bewertung umfasst den gesamten Ordner `src/`:
- `src/main.rs`
- `src/game/*`
- `src/views/*`
- `src/helper/*`
- `src/level.rs`
- `src/world.rs`

Ziel: Stärken, Risiken und konkretes Verbesserungspotential mit Priorisierung.

---

## Gesamturteil
Die Architektur ist fuer ein wachsendes Bevy-Spiel bereits **gut strukturiert** und klar in Schichten getrennt:
- Bootstrap/State-Einstieg in `src/main.rs`
- Laufzeit-Gameplay in `src/game/*`
- UI-Views in `src/views/*`
- Querschnitts-Helper in `src/helper/*`
- Datengetriebene Assets in `src/level.rs` und `src/world.rs`

Besonders positiv ist der plugin-basierte Aufbau und die feingranulare Systemorganisation. Das groesste Optimierungspotential liegt in der **Entkopplung von Typen/Ressourcen aus `main.rs`**, in **stabileren App-SystemSets** und in der **Vereinheitlichung von Asset-I/O und Fehlerbehandlung**.

---

## Stärken

### 1) Saubere vertikale Aufteilung (UI / Gameplay / Helper / Data)
- `src/views/*` und `src/game/*` sind logisch getrennt.
- `src/level.rs` und `src/world.rs` setzen konsequent auf datengetriebene Inhalte.
- `src/helper/*` kapselt Persistenz und Querschnittsthemen (Audio, i18n, Keybindings, Fonts).

### 2) Gute Bevy-Plugin-Nutzung
- `ViewsPlugin` (`src/views/mod.rs`) und `GameViewPlugin` (`src/game/game_view.rs`) geben klare Entry-Points.
- `MainViewPlugin` (`src/views/main_view.rs`) entlastet `src/main.rs` deutlich.

### 3) Gute Systemgranularitaet im Gameplay
- `src/game/systems/*` ist in `gameplay`, `presentation`, `maintenance`, `setup_spawn` unterteilt.
- Viele kleine Systeme sind leichter testbar und einfacher zu sortieren.

### 4) Datenmodell und Validierung vorhanden
- `src/level.rs` validiert Entity-Typen (`states.default`, nicht-leere states).
- Overrides (`serde(flatten)`) ermoeglichen flexible Konfiguration pro Entity.

### 5) Utility-Layer mit praktischer Produktivitaet
- Persistenz von Sprache/Settings/Bindings ist vorhanden.
- i18n-Mechanik (`LocalizedText`) ist durchgaengig nutzbar.

---

## Risiken und Schwachstellen

### A) Hohe Kopplung an `crate::`-Typen aus `main.rs` (hoch)
Befund:
- `src/views/main_view.rs` referenziert viele Typen/Konstanten aus `main.rs` (`MenuSelection`, `ExitConfirmModalState`, `MENU_ITEMS`, Buttons etc.).

Risiko:
- `main.rs` bleibt trotz ausgelagerter Systeme semantisch ueberladen.
- Refactorings sind teurer, weil viele View-Dateien implizit von `main.rs`-Interna abhaengen.

### B) System-Ordering ueber viele Einzelaufrufe statt strukturierte Sets (mittel)
Befund:
- In `src/game/game_view.rs` und `src/views/main_view.rs` erfolgt die Orchestrierung direkt in langen `add_systems`-Ketten.

Risiko:
- Bei Wachstum steigt die Gefahr von Reihenfolge-Regressions.
- Debugging von Scheduling-Konflikten wird schwerer.

### C) Duplizierte Asset-I/O-Muster in `level.rs` und `world.rs` (mittel)
Befund:
- Aehnliche Lade- und Reader-Logik existiert in beiden Dateien.

Risiko:
- Doppelte Wartung bei Fehlerbehandlung/Performance-Anpassungen.

### D) `main.rs` enthaelt weiterhin viele Domaintypen (mittel)
Befund:
- States, MenuActions, Resources, UI-Components fuer MainMenu liegen weiterhin in `src/main.rs`.

Risiko:
- Der Bootstrap-Entry bleibt als Sammelstelle fuer nicht-bootstrap Verantwortlichkeiten.

### E) Inkonsistente Fehlerbehandlung/Logging (niedrig-mittel)
Befund:
- Teilweise `warn!`, teilweise `eprintln!`, teilweise Rueckgabe als `String`.

Risiko:
- Uneinheitliche Diagnostik im Betrieb.

---

## Verbesserungspotential (konkret)

## P0 (als naechstes)

### 1) App-Model aus `main.rs` herausziehen
Empfehlung:
- Neues Modul z. B. `src/app_model.rs` oder `src/app_state.rs` anlegen.
- Dort hin verschieben:
  - `AppState`
  - MainMenu-Actions
  - Menu-Resources (`MenuSelection`, `ExitConfirmModalState`)
  - MainMenu-Components (`MainMenuEntity`, `MenuButton`, etc.)
  - Konfigurationen wie `MENU_ITEMS`, `EXIT_CONFIRM_ITEMS`

Nutzen:
- `main.rs` wird wirklich Bootstrap-only.
- Views/Game koennen ein neutrales Modul statt `main.rs` importieren.

### 2) `MainViewPlugin` API sauber begrenzen
Empfehlung:
- In `src/views/main_view.rs` alles intern halten, was nicht extern registriert werden muss.
- Kommentare aktualisieren (z. B. Verweise auf "called from main.rs" entfernen, wenn nicht mehr zutreffend).

Nutzen:
- Bessere Kapselung und weniger Leaks in die Crate-Oeffentlichkeit.

## P1 (kurzfristig)

### 3) SystemSets fuer Views und Gameplay einfuehren
Empfehlung:
- Eigene Sets definieren, z. B.:
  - `MainMenuSet::Input`
  - `MainMenuSet::Action`
  - `MainMenuSet::Visual`
  - `MainMenuSet::Modal`
- Aehnlich in `GameViewPlugin` fuer Gameplay/Presentation/Maintenance.

Nutzen:
- Klarere Reihenfolge und robustere Erweiterbarkeit.

### 4) Shared Asset-Loader Utility bauen
Empfehlung:
- Gemeinsame Funktionen fuer `read_asset_text_from_server` in ein Modul wie `src/helper/asset_io.rs` verschieben.
- `level.rs` und `world.rs` darauf umstellen.

Nutzen:
- Weniger Duplikatcode, konsistente Fehlertexte, zentrale Verbesserungen moeglich.

### 5) Fehler-/Logging-Strategie vereinheitlichen
Empfehlung:
- Konsistent `thiserror` + `tracing`/`log` nutzen.
- Keine `String`-Fehler in Ressourcen, wo typisierte Errors moeglich sind.

Nutzen:
- Bessere Diagnose in Development und Builds.

## P2 (mittelfristig)

### 6) View-spezifische Komponenten lokal kapseln
Empfehlung:
- Komponenten, die nur in MainMenu gebraucht werden, in `views/main_view.rs` oder Untermodul `views/main_view/types.rs` halten.

Nutzen:
- Kleinere API-Oberflaeche und weniger globale Namensabhaengigkeiten.

### 7) Architekturtests / Smoke-Checks erweitern
Empfehlung:
- Leichte Architektur-Regressionstests (z. B. Plugin-Registrierung smoke tests, Resource-Defaults, State-Transitions).

Nutzen:
- Sicherere Refactors bei wachsender Systemzahl.

---

## Modulbewertung im Ueberblick

- `src/main.rs`: **B+**
  - Positiv: schlanker als zuvor, guter Bootstrap.
  - Potenzial: noch zu viele App-/Menu-Typen lokal.

- `src/views/*`: **A-**
  - Positiv: klare View-Plugins, MainMenu-Logik zentral in `main_view`.
  - Potenzial: starke Abhaengigkeit von Typen aus `main.rs` reduzieren.

- `src/game/*`: **A-**
  - Positiv: sehr gute Zerlegung in Systemdomaenen, nachvollziehbares Scheduling.
  - Potenzial: SystemSets/Ordering robuster machen.

- `src/helper/*`: **B+**
  - Positiv: nuetzliche Querschnittsmodule.
  - Potenzial: Logging/Fehlerkonsistenz, evtl. klarere Trennung "pure helper" vs. "persisted settings".

- `src/level.rs` + `src/world.rs`: **B+**
  - Positiv: datengetrieben, valide Basisstrukturen, brauchbare Tests.
  - Potenzial: gemeinsame Asset-I/O extrahieren, Fehlerstrategie vereinheitlichen.

---

## Empfohlene Refactor-Reihenfolge
1. `app_model`/`app_state` Modul extrahieren (P0).
2. `MainViewPlugin` Kommentare/API bereinigen (P0).
3. View- und Gameplay-SystemSets einfuehren (P1).
4. Gemeinsames `helper/asset_io.rs` erstellen und `level/world` migrieren (P1).
5. Logging/Error-Standardisierung (P1/P2).

---

## Fazit
Die aktuelle Architektur ist fuer den Projektstand solide und skalierbar. Mit wenigen gezielten Schritten (vor allem Entkopplung von `main.rs`-Typen und strukturierteres Scheduling) laesst sich die Wartbarkeit deutlich steigern, ohne das bestehende Gameplay- und View-Modell aufzubrechen.

