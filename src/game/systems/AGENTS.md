# Systems Reference — src/game/systems

Kurzüberblick

Diese Datei dokumentiert die Systems im Ordner `src/game/systems`. Ziel ist eine kurze, leicht verdauliche Referenz pro System mit Zweck, wichtigsten Signaturen (Queries / Resources), Seiteneffekten und Hinweisen für Wartung oder Tests.

Checklist (wie diese Datei erstellt wurde)
- Dateien aufgelistet: `mod.rs`, `plugin.rs`, und alle `*_system.rs` Dateien
- Für jede Datei: kurze Beschreibung, Hauptfunktion(en), verwendete Components/Resources, wichtige Seiteneffekte, TODOs

Konvention der Einträge
- Datei: Pfad/Dateiname
- Zweck: 1–2 Sätze
- Öffentliche Funktion(en): Name und Kurzbeschreibung
- Queries / Resources: Res<> / Query<> Signaturen (vereinfachte Darstellung)
- Verwendete Components / Runtime-Components / Tags
- Seiteneffekte / Mutationen: Commands, Despawn, Insert, etc.
- Hinweise / TODOs: Tests, potentielle Verbesserungen

---

## Dateien und Systeme

### `plugin.rs` / `SystemsPlugin`
- Zweck: Fügt alle Gameplay-Systeme in sinnvolle `SystemSet`s ein und registriert sie nur im `AppState::GameView`.
- Öffentliche Struktur/Funktion: `SystemsPlugin` (impl `Plugin`)
- Wichtige Sets: `GameplaySet::Input`, `Ai`, `Physics`, `Grounding`, `Projectile`, `Finalize`.
- Registrierte Systeme (in Reihenfolge):
  - `player_control_system` (Input)
  - `enemy_random_patrol_system` (Ai)
  - `gravity_integration_system` (Physics)
  - `movement_resolution_system` (Physics)
  - `grounding_evaluation_system` (Grounding)
  - `projectile_collision_system` (Projectile)
  - `track_previous_transform_system` (Finalize)
- Seiteneffekte: Konfiguration von Set-Chaining und Run-If (State-abhängig)
- Hinweis: `SystemsPlugin` ist zentrale Stelle um Ausführungsreihenfolge zu ändern; beim Refactoring Sets beibehalten.

### `player_control_system.rs` — `player_control_system`
- Zweck: Liest Input und steuert Spieler-bezogene Movement-States (axis, jump).
- Öffentliche Funktion(en): `player_control_system`
- Signatur (vereinfacht):
  - `keyboard: Res<ButtonInput<KeyCode>>`
  - `key_bindings: Res<KeyBindings>`
  - `players: Query<(&ControlledMovement, &mut Gravity, &mut RigidBody), With<PlayerTag>>`
- Verwendete Components/Tags: `ControlledMovement`, `Gravity`, `RigidBody`, `PlayerTag`
- Seiteneffekte: Mutiert `RigidBody.velocity`, ändert `Gravity.grounded` beim Springen
- TODO / Tests: Unit-Test für KeyBinding -> Velocity-Mapping; Integrationstest für Sprung (nur wenn grounded)

### `enemy_random_patrol_system.rs` — `enemy_random_patrol_system`
- Zweck: Steuerung einfacher patrouillierender Gegner mit zufälligen Intervallen.
- Öffentliche Funktion(en): `enemy_random_patrol_system`
- Signatur (vereinfacht):
  - `commands: Commands`
  - `time: Res<Time>`
  - `enemies: Query<(Entity, &mut AutoMovement, &mut RigidBody, Option<&mut PatrolState>), With<EnemyTag>>`
- Verwendete Components/Tags: `AutoMovement`, `RigidBody`, `PatrolState` (runtime), `EnemyTag`
- Seiteneffekte: Kann `PatrolState` per `commands.entity().insert(...)` anlegen; verändert `RigidBody.velocity.x`.
- Hinweise: Default-Speed-Fallback (`DEFAULT_PATROL_SPEED`) und Pseudozufall via `PatrolState.next_rand()`; deterministic tests möglich durch PatrolState-Seed.

### `gravity_integration_system.rs` — `gravity_integration_system`
- Zweck: Wendet die Welt-Gravitation und optionale Zusatzbeschleunigungen auf Entities an.
- Öffentliche Funktion(en): `gravity_integration_system`
- Signatur (vereinfacht):
  - `time: Res<Time>`
  - `world_gravity: Res<avian2d::prelude::Gravity>`
  - `entities: Query<(&Gravity, &mut RigidBody)>`
- Verwendete Components: `Gravity`, `RigidBody`
- Seiteneffekte: Mutiert `RigidBody.velocity`; überspringt statische Bodies (`is_static()`)
- Hinweise: Wendet linear damping an (clamped multiplikativer Dämpfungsfaktor). Kann leicht in Fixed-step-System verschoben werden, falls nötig.

### `movement_resolution_system.rs` — `movement_resolution_system`
- Zweck: Löst Bewegungskollisionen zwischen beweglichen Objekten (Mover) und Blockern; führt AABB-basierte Kollisionserkennung und -auflösung durch und aktualisiert Grounding-Informationen.
- Öffentliche Funktion(en): `movement_resolution_system`
- Signatur (vereinfacht):
  - `commands: Commands`
  - `time: Res<Time>`
  - `movers: Query<(Entity, &mut Transform, &Collider, &mut RigidBody, &Gravity, Option<&mut GroundingState>), Without<Blocking>>`
  - `blockers: Query<(Entity, &Transform, &Collider, Option<&RigidBody>, Option<&PreviousTransform>), With<Blocking>>`
- Verwendete Components/Runtime-Components: `Collider`, `RigidBody`, `Gravity`, `GroundingState`, `PreviousTransform`, `Blocking`
- Wichtige Hilfsfunktionen (intern): `resolve_axis`, `blocker_step_velocity`, `rectangle_half_extents`, `aabb_from_rect` und `Aabb::overlaps`.
- Verhalten:
  - Trennt X- und Y-Achse und löst Überschneidungen jeweils separat.
  - Erlaubt horizontales Durchschreiten, wenn Kontakt ground-like ist (obenauf, schmale vertikale Überlappung).
  - Setzt `rigid_body.velocity` Komponenten auf `0.0` bei Kollision
  - Aktualisiert oder fügt `GroundingState` per `commands.entity().insert(...)` hinzu
- Seiteneffekte: Transform-Änderungen, Insert von `GroundingState`, Mutationen von `RigidBody.velocity`.
- Hinweise: `MAX_GROUND_ANGLE_DEG` wird zur Bestimmung von Ground-Normalen-Schwelle genutzt; intern AABB-basierte TOI/Overlap-Logik (keine physikalische Penetration-Resolving außerhalb AABB).

### `grounding_evaluation_system.rs` — `grounding_evaluation_system`
- Zweck: Wertet gesammelte Kontaktinformationen (`GroundingState`) aus und entscheidet, ob ein Entity als `grounded` gilt.
- Öffentliche Funktion(en): `grounding_evaluation_system`
- Signatur (vereinfacht):
  - `time: Res<Time>`
  - `world_gravity: Res<avian2d::prelude::Gravity>`
  - `entities: Query<(&mut Gravity, &RigidBody, &mut GroundingState)>`
- Verwendete Components/Runtime-Components: `Gravity`, `RigidBody`, `GroundingState`
- Verhalten:
  - Berechnet required support force basierend auf Mass * Gravity * SUPPORT_THRESHOLD
  - Vergleicht mit `support_force` und setzt `gravity.grounded` entsprechend
  - Implementiert Hysterese (`GROUND_EXIT_HYSTERESIS_SEC`) bevor `grounded` deaktiviert wird
- Seiteneffekte: Mutiert `Gravity.grounded` und setzt ggf. `GroundingState.support_velocity` zurück.

### `projectile_collision_system.rs` — `projectile_collision_system`
- Zweck: Ermittelt Kollisionen für Projektile gegen Ziele per swept-AABB (time-of-impact) und wendet Schaden an / despawnt Projektile.
- Öffentliche Funktion(en): `projectile_collision_system`
- Signatur (vereinfacht):
  - `commands: Commands`
  - `time: Res<Time>`
  - `projectiles: Query<(Entity, &Transform, &Collider, &RigidBody, &Projectile)>`
  - `targets: Query<(Entity, &Transform, &Collider, Option<&Blocking>, Option<&Damageable>, Option<&RigidBody>, Option<&Team>)>`
  - `teams: Query<&Team>`
  - `mut health_query: Query<&mut Health>`
- Verwendete Components/Runtime-Components: `Projectile`, `Collider`, `RigidBody`, `Blocking`, `Damageable`, `Health`, `Team`
- Wichtiges Verhalten:
  - Berechnet relative Motion (projectile - target)
  - Nutzt `swept_aabb_toi` und `ray_axis_times` um TOI zu finden
  - Wählt bestes Trefferziel (TOI, Distanz, Priorität blocking vs damageable)
  - Wendet Standard-Schaden (`DEFAULT_PROJECTILE_DAMAGE`) an, falls Ziel damageable
  - Despawnt das Projectile nach Treffer
- Seiteneffekte: Mutiert `Health` via `health.damage(...)`, ruft `commands.entity(...).despawn()` auf.
- Hinweise: Team-Check (`NEUTRAL_TEAM`) verhindert Friendly-Fire; TOI-Epsilon (`TOI_EPSILON`) steuert Toleranzen bei Auswahl.

### `track_previous_transform_system.rs` — `track_previous_transform_system`
- Zweck: Speichert die vorherige Position von Blockern (Entities mit `Blocking`) in `PreviousTransform` für Velocity-Abschätzung.
- Öffentliche Funktion(en): `track_previous_transform_system`
- Signatur (vereinfacht):
  - `commands: Commands`
  - `blockers: Query<(Entity, &Transform, Option<&mut PreviousTransform>), With<Blocking>>`
- Verwendete Components: `Blocking`, `PreviousTransform`
- Seiteneffekte: Fügt `PreviousTransform` mittels `commands.entity(...).insert(...)` hinzu oder aktualisiert `previous.position`.
- Hinweise: Wird in `Finalize` SystemSet registriert (Plugin), sodass vorherige Systeme Positionsänderungen abgeschlossen haben sollten.

### `mod.rs`
- Zweck: Re-exportiert Systems und das `SystemsPlugin`-Plugin. Keine Logik, aber nützlich als Index.

---

Weitere Hinweise und TODOs
- Tests: Viele Systeme sind deterministisch und eignen sich gut für Unit-Tests (z. B. `swept_aabb_toi`, `ray_axis_times`, `movement resolve`-Grenzfälle). Erwäge kleine Test-Utilities zur Erstellung einfacher Entities mit minimalen Komponenten.
- Performance: `movement_resolution_system` und `projectile_collision_system` iterieren über viele Pairings; falls Skalierung ein Thema wird, erwäge räumliche Partitionierung (Zellen/Quadtree) oder Broadphase.
- Robustheit: Einige Funktionen erwarten Rectangle-Colliders; andere Collider-Shapes werden aktuell ignoriert (`rectangle_half_extents` returns Option). Entweder dokumentieren oder ausbauen.

Wenn du möchtest, kann ich:
- kurze Beispiele zeigen, wie man ein System testet (Unit-Test für `swept_aabb_toi` z.B.)
- automatische Extraktion der Query-Signaturen als JSON erzeugen
- die AGENTS.md in kleinere thematische Dateien aufteilen

---

Ende der Systems-Referenz

