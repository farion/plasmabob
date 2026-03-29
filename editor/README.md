# PlasmaBob Level Editor

Eigenstaendiger Leveleditor fuer PlasmaBob auf Basis von Rust und Bevy.

## Funktionen
- Levelauswahl aus `../assets/levels/*.json`
- Rendering von Background und Entities aus `../assets/entity_types`
- Entity-Auswahl mit roter Umrandung
- Drag and Drop mit der Maus
- `A` oeffnet ein Menue zum Hinzufuegen neuer Entities
- `D` entfernt die aktuell ausgewaehlte Entity
- `Ctrl+S` speichert das aktuelle Level in die passende JSON-Datei
- Kurze Speichern-Bestaetigung unten rechts
- Mausrad zum Zoomen, mittlere Maustaste zum Verschieben der Kamera

## Starten
```powershell
cd editor
cargo run
```

## Bedienung
- **Linksklick**: Entity auswaehlen
- **Linke Maustaste halten und ziehen**: Ausgewaehlte Entity verschieben
- **A**: Menue zum Hinzufuegen eines Entity-Typs oeffnen oder schliessen
- **D**: Ausgewaehlte Entity entfernen
- **Ctrl+S**: Level speichern
- **Mausrad**: Zoom
- **Mittlere Maustaste ziehen**: Kamera verschieben
