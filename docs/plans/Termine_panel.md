# Plan: "Termine"-Panel — Geburtstage & Jahrestage als Liste

## Context

Projekt "Soziale Beziehungen" (Tauri+Svelte+Rust+SQLite) erfasst Personen + Beziehungen als Graph. Geplante Personendaten-Erweiterung (`docs/plans/2026-06-22-extended-person-data-design.md`) fügt `birth_date` und `known_since` zu `people` hinzu. Daraus ergibt sich natürlicher Mehrwert: eine Übersicht über anstehende Geburtstage und "Kennengelernt"-Jahrestage, damit der Nutzer nichts vergisst.

Brainstorming-Ergebnis (Session 2026-06-22): Scope bewusst klein gehalten — **nur Anzeige**, keine Notifications, kein Kontakt-Tracking (das sind eigene, spätere Subprojekte). Familien-Jahrestage (z.B. Hochzeitstag) sind **nicht** im Scope, da `relationships` kein Datumsfeld dafür hat und Nutzer entschieden hat, das nicht jetzt nachzurüsten.

## Entscheidung: rein client-seitig

Kein neuer Backend-Command. `birth_date`/`known_since` sind bereits im `people`-Array, das `App.svelte` über `getGraph()` lädt. Datums-Logik (nächstes Vorkommen, Sortierung) läuft in JS — kein Rust-Code, keine Schema-Änderung über die bereits geplante Migration hinaus.

## Komponenten

### `src/lib/dates.js` (neu)

```js
export function getUpcomingEvents(people) {
  // Für jede Person mit gültigem birth_date: Eintrag
  //   { personId, type: 'Geburtstag', icon: '🎂', nextDate, extra: `wird ${turningAge}` }
  // Für jede Person mit gültigem known_since: Eintrag
  //   { personId, type: 'Kennengelernt', icon: '🤝', nextDate, extra: `${years} Jahre bekannt` }
  // nextDate = Monat/Tag des Datums, Jahr = aktuelles Jahr; falls < heute, +1 Jahr
  // ungültige/fehlende Daten werden übersprungen (kein Fehler)
  // Rückgabe aufsteigend sortiert nach nextDate
}
```

- Reine Funktion, kein State — leicht testbar
- Schaltjahr-Edge-Case (29. Februar): im Plan nicht extra behandelt, JS `Date`-Arithmetik regelt das implizit (fällt in Nicht-Schaltjahren auf 1. März o.ä. — akzeptabel für diesen Scope)

### `src/lib/UpcomingPanel.svelte` (neu)

- Liste aller Events aus `getUpcomingEvents(people)`, chronologisch
- Pro Zeile: Icon, Personenname, Datum (TT.MM.), Countdown ("in 12 Tagen") oder "heute"/"morgen", Zusatzinfo (Alter/Jahre)
- Klick auf Zeile → `onSelectPerson(personId)` Callback (App.svelte setzt `selectedPersonId`, öffnet bestehendes `PersonForm`)
- `onClose`-Callback wie bei `PersonForm`/`SettingsPanel`

### `App.svelte` Integration

- Neuer State `showUpcoming = $state(false)` (analog `showNewPersonForm`/`showSettings`)
- Neuer Toolbar-Button "Termine" neben "+ Person"/"Verbinden"/"Einstellungen"
- Sidebar-Bedingung erweitert: `{#if showNewPersonForm || selectedPerson || selectedEdge || showSettings || showUpcoming}` → `<UpcomingPanel {people} onSelectPerson={...} onClose={...} />`
- Klick auf Event-Zeile schließt `showUpcoming`, setzt `selectedPersonId`

## Betroffene Dateien

- `src/lib/dates.js` (neu)
- `src/lib/UpcomingPanel.svelte` (neu)
- `src/App.svelte` (Toolbar-Button + State + Panel-Routing)

Keine Backend-/Migrations-Änderung über das bereits geplante `birth_date`/`known_since` hinaus.

## Verifikation

- `pnpm tauri dev` starten, Personen mit `birth_date`/`known_since` anlegen (sobald Migration umgesetzt ist)
- Toolbar-Button "Termine" öffnet Liste, korrekt sortiert nach nächstem Datum
- Klick auf Eintrag öffnet `PersonForm` der richtigen Person
- Manuell: Person mit Geburtstag heute/morgen → Countdown korrekt ("heute"/"morgen"/"in N Tagen")
- Person ohne `birth_date`/`known_since` taucht nicht in Liste auf
