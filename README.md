# Soziale Beziehungen

Desktop-App (Tauri 2 + Svelte 5 + Rust + SQLite) zum Erfassen und
visualisieren von sozialen Beziehungen als interaktiver Graph (Cytoscape.js).

## Setup

Voraussetzungen: Rust-Toolchain (`cargo`), Node.js, und die üblichen
Tauri-Systemabhängigkeiten für dein Betriebssystem (siehe
https://v2.tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm tauri dev
```

Die SQLite-Datenbank wird automatisch im OS-spezifischen App-Datenordner
angelegt (z.B. unter Windows `%APPDATA%\social-graph\social-graph.db`) und
beim ersten Start via `sqlx::migrate!` initialisiert.

## Build

```bash
pnpm tauri build
```

## Architektur

- **src-tauri/migrations/** – SQL-Migrationen (sqlx)
- **src-tauri/src/models.rs** – Person, Relationship, GraphData Structs
- **src-tauri/src/db.rs** – DB-Pool-Initialisierung
- **src-tauri/src/commands.rs** – Tauri Commands (CRUD für People/Relationships, get_graph)
- **src/lib/api.js** – JS-Wrapper um `invoke()`
- **src/lib/Graph.svelte** – Cytoscape-Canvas, Klick-Handling
- **src/lib/PersonForm.svelte** / **EdgeForm.svelte** – Detailpanels
- **src/App.svelte** – State-Management, verbindet alles

## Beziehungsmodell

Beziehungen sind **ungerichtet**: "A kennt B" = "B kennt A". Intern wird
beim Speichern immer `from_id < to_id` normalisiert, damit es pro Paar nur
einen Datensatz gibt (verhindert doppelte Kanten A→B und B→A).

## Bedienung

- **+ Person**: neue Person anlegen (Name, Notiz, Farbe)
- **Verbinden**: Modus aktivieren, dann zwei Knoten im Graph antippen →
  legt eine Beziehung mit Default-Werten an (Art: "kennt", Stärke 3),
  die danach im Edge-Panel weiter bearbeitet werden kann
- Klick auf einen **Knoten** öffnet das Bearbeiten-Panel für die Person
- Klick auf eine **Kante** öffnet das Bearbeiten-Panel für die Beziehung
- Klick auf die **leere Fläche** schließt alle Panels

## Mögliche nächste Schritte

- Filtern/Suche nach Personen oder Beziehungsart
- Cluster-Erkennung (z.B. via Graph-Algorithmen in Rust)
- Export als PNG/JSON
- Tags/Gruppen für Personen (z.B. "Familie", "Arbeit", "Hobby")
