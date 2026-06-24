# Kontakt-Verlauf — Design

**Datum:** 2026-06-24
**Status:** Approved

## Ziel

Der Verlauf von "Zuletzt kontaktiert" soll festgehalten werden (jeder Kontakt
bleibt erhalten, nicht nur der letzte) und in der Detailansicht (`PersonModal`)
als aufklappbare Timeline ansprechend angezeigt werden.

## Aktueller Stand

`people` hat zwei Spalten `last_contact_type` / `last_contact_date` (Migration
0008). Sie werden bei jedem Setzen via `set_last_contact` **überschrieben** — es
gibt also keinen Verlauf. Die Spalten werden in `PERSON_COLUMNS` mitgelesen und
ans Frontend (Graph, Liste, Modal) geliefert.

## Architektur-Entscheidung

**Neue Tabelle `contact_events` (jeder Kontakt = 1 Zeile) + `people.last_contact_*`
als Cache des jüngsten Events.**

Begründung: Graph, Personenliste und `PERSON_COLUMNS` lesen weiterhin
`person.last_contact_*`. Diese Spalten bleiben erhalten, werden aber neu
definiert als denormalisierter Cache (= Event mit höchstem `contact_date`). So
bleibt der bestehende Code unverändert; nur das Schreiben des Kontakts ändert
sich. Kleinster Blast-Radius.

Cache wird bei jedem Hinzufügen/Löschen eines Events neu berechnet.

## Datenmodell pro Event

- `contact_type` — Kontaktart (`in_person` / `messenger` / `call` / `email`)
- `contact_date` — Datum (`YYYY-MM-DD`)
- `note` — optionaler Freitext (z.B. "Geburtstag", "kurz telefoniert")

## Verhalten

- Einträge: **hinzufügen + löschen**. Kein nachträgliches Bearbeiten.
- Anzeige: **aufklappbare Timeline** in der Detailansicht.

---

## 1. Datenbank — Migration `src-tauri/migrations/0009_contact_events.sql`

```sql
CREATE TABLE contact_events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id    INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    contact_type TEXT NOT NULL,
    contact_date TEXT NOT NULL,          -- 'YYYY-MM-DD'
    note         TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_contact_events_person ON contact_events(person_id, contact_date DESC);

-- vorhandenen letzten Kontakt als ersten Event-Eintrag übernehmen
INSERT INTO contact_events (person_id, contact_type, contact_date)
SELECT id, last_contact_type, last_contact_date FROM people
WHERE last_contact_type IS NOT NULL AND last_contact_date IS NOT NULL;
```

`people.last_contact_type` / `last_contact_date` bleiben bestehen = Cache des
jüngsten Events.

## 2. Backend (Rust)

### `models.rs`

Neues Struct:

```rust
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ContactEvent {
    pub id: i64,
    pub person_id: i64,
    pub contact_type: String,
    pub contact_date: String,
    pub note: Option<String>,
    pub created_at: String,
}
```

### `commands.rs`

Interner Helper zum Neuberechnen des Caches:

```sql
UPDATE people SET
  last_contact_type = (SELECT contact_type FROM contact_events
                       WHERE person_id = ? ORDER BY contact_date DESC, id DESC LIMIT 1),
  last_contact_date = (SELECT contact_date FROM contact_events
                       WHERE person_id = ? ORDER BY contact_date DESC, id DESC LIMIT 1)
WHERE id = ?;
```

Commands:

- `list_contact_events(person_id) -> Vec<ContactEvent>`
  `SELECT ... FROM contact_events WHERE person_id = ? ORDER BY contact_date DESC, id DESC`
- `add_contact_event(person_id, contact_type, contact_date, note) -> Person`
  Insert, dann Cache neu berechnen, aktualisierte `Person` zurückgeben
  (via `attach_image` wie bei den übrigen Person-Commands).
- `delete_contact_event(event_id) -> Person`
  `person_id` des Events ermitteln, Zeile löschen, Cache neu berechnen,
  aktualisierte `Person` zurückgeben.

Der bisherige Command `set_last_contact` entfällt (wird durch
`add_contact_event` ersetzt).

### `lib.rs`

Die drei neuen Commands im `invoke_handler` registrieren; `set_last_contact`
dort entfernen.

## 3. Frontend (Svelte)

### `src/lib/api.js`

- Neu: `listContactEvents(personId)`, `addContactEvent(personId, contactType, contactDate, note)`,
  `deleteContactEvent(eventId)`.
- Entfernen: `setLastContact`.
- Bleiben: `contactTypes`, `contactTypeLabel`.

### Neue Komponente `src/lib/ContactTimeline.svelte`

Props: `{ person, onChange }`.

- **Summary-Zeile:** letzter Kontakt als `contactTypeLabel – Datum` (oder "—"),
  Button `+` (neuen Kontakt anlegen), Button `▾` (Timeline auf-/zuklappen).
- **Aufgeklappt:** vertikale Timeline, jüngster Eintrag oben. Pro Eintrag:
  Typ-Icon/Dot, Label, Datum, Notiz (falls vorhanden), `×` zum Löschen.
- **Add-Flow:** `+` → Typ-Pills auswählen → Datum-Input (Default heute) +
  optionales Notiz-Feld → speichern.
- Lädt eigene Event-Liste via `listContactEvents` beim Öffnen und nach jedem
  Add/Delete. Ruft nach Mutationen `onChange()`, damit Graph und Person-Cache
  im Modal aktualisiert werden.
- Typ-Icons (Emoji, keine Asset-Abhängigkeit): 📞 call · 💬 messenger ·
  ✉ email · 🤝 in person.

### `src/lib/PersonModal.svelte`

- Entfernen: State `contactOpen` / `pendingContactType`, Funktionen
  `openContactPicker` / `pickContactType` / `saveContactDate`, Import von
  `setLastContact`, `contactTypes`, `contactTypeLabel` (sofern nur hier genutzt).
- Im `<dd>` von "Zuletzt kontaktiert" den bisherigen Inline-Block durch
  `<ContactTimeline {person} {onChange} />` ersetzen.
- Verwaiste Styles (`.pills`, `.pill`, `.mini-btn`) in die neue Komponente
  verschieben.

## Testing / Verifikation

- Migration läuft sauber, vorhandener `last_contact` erscheint als erster
  Timeline-Eintrag.
- Kontakt hinzufügen mit/ohne Notiz → erscheint in Timeline, Summary-Zeile +
  Graph zeigen jüngsten Kontakt.
- Kontakt löschen → verschwindet; Summary/Cache fällt auf nächstjüngeren bzw.
  "—" zurück.
- Person löschen → zugehörige `contact_events` per `ON DELETE CASCADE` weg.
