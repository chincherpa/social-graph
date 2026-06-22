# Design: Erweiterte Personendaten & Familienbeziehungen

**Datum**: 2026-06-22  
**Status**: Design validiert, implementierung pending  

---

## Anforderungen

Der Nutzer möchte für jede Person folgende Informationen ablegen können:

- Name, Vorname, Spitzname
- Adresse
- Geburtsdatum
- Datum Kennengelernt
- Familienmitglieder (Namen, beliebig viele mit Beziehungstyp)
- Arbeitgeber
- Kennt-wen-Links (bestehende Relationships erweitern)

## Design-Entscheidungen

### 1. Familienbeziehungen

- **Struktur**: Separate `family_members`-Tabelle (Option A)
- **Verlinkung**: Familienmitglieder sind auch reguläre `people`-Einträge in der DB
- **Graph-Integration**: Wenn Person A "verheiratet mit Person B" → automatisch neue Person B erstellt + `relationships`-Eintrag vom Typ "verheiratet" (Familie ist im Graph sichtbar)

### 2. Normalisierung

- `family_members.person_id < family_members.family_id` wird **nicht** erzwungen (Beziehung ist gerichtet: Person A hat Familie B)
- Aber: `UNIQUE(person_id, family_id)` verhindert Duplikate

---

## Datenbank-Schema

### Migration: `0002_extended_people.sql`

```sql
-- Erweiterte Personendaten
CREATE TABLE IF NOT EXISTS people (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name    TEXT,
    last_name     TEXT NOT NULL,
    nickname      TEXT,
    birth_date    TEXT,  -- ISO 8601 Format
    known_since   TEXT,  -- ISO 8601 Format
    address       TEXT,
    employer      TEXT,
    color         TEXT NOT NULL DEFAULT '#3b82f6',
    note          TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Familienmitglieder: Verlinkt Personen als Familie
CREATE TABLE IF NOT EXISTS family_members (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    family_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,  -- 'Ehepartner', 'Kind', 'Mutter', 'Vater', 'Geschwister', etc.
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(person_id, family_id),
    CHECK(person_id != family_id)
);

CREATE INDEX IF NOT EXISTS idx_family_person ON family_members(person_id);
CREATE INDEX IF NOT EXISTS idx_family_member ON family_members(family_id);
```

**Migration-Strategie:**
- Alte `people`-Tabelle wird umbenannt zu `people_old`
- Neue Tabelle mit erweitertem Schema erstellt
- Daten aus `people_old` migriert (first_name leer, last_name = alte name, andere Felder NULL)
- `people_old` gelöscht

---

## Rust Backend

### Neue Datenstrukturen (`models.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub color: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewPersonPayload {
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub color: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonPayload {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub color: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FamilyMember {
    pub id: i64,
    pub person_id: i64,
    pub family_id: i64,
    pub relation_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonWithFamily {
    pub person: Person,
    pub family_members: Vec<FamilyMemberDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FamilyMemberDetail {
    pub id: i64,
    pub family_id: i64,
    pub relation_type: String,
    pub family_last_name: String,  // für einfache Anzeige
    pub family_first_name: Option<String>,
}
```

### Neue Commands (`commands.rs`)

```rust
#[tauri::command]
pub async fn add_person(payload: NewPersonPayload, state: tauri::State<'_, AppState>) -> Result<Person, String>

#[tauri::command]
pub async fn update_person(payload: UpdatePersonPayload, state: tauri::State<'_, AppState>) -> Result<Person, String>

#[tauri::command]
pub async fn get_person(id: i64, state: tauri::State<'_, AppState>) -> Result<PersonWithFamily, String>

#[tauri::command]
pub async fn add_family_member(
    person_id: i64,
    family_id: i64,
    relation_type: String,
    state: tauri::State<'_, AppState>
) -> Result<FamilyMember, String>
// Bei Erfolg: Auch neue Relationship vom Typ relation_type erstellen

#[tauri::command]
pub async fn remove_family_member(person_id: i64, family_id: i64, state: tauri::State<'_, AppState>) -> Result<(), String>
// Bei Erfolg: Auch Relationship löschen

#[tauri::command]
pub async fn get_family(person_id: i64, state: tauri::State<'_, AppState>) -> Result<Vec<FamilyMemberDetail>, String>
```

### DB-Funktionen (`db.rs`)

```rust
pub async fn add_person(pool: &SqlitePool, payload: &NewPersonPayload) -> Result<Person, String>
pub async fn update_person(pool: &SqlitePool, payload: &UpdatePersonPayload) -> Result<Person, String>
pub async fn get_person_by_id(pool: &SqlitePool, id: i64) -> Result<Person, String>
pub async fn get_person_with_family(pool: &SqlitePool, id: i64) -> Result<PersonWithFamily, String>
pub async fn add_family_member(pool: &SqlitePool, person_id: i64, family_id: i64, relation_type: &str) -> Result<FamilyMember, String>
pub async fn remove_family_member(pool: &SqlitePool, person_id: i64, family_id: i64) -> Result<(), String>
pub async fn get_family_members(pool: &SqlitePool, person_id: i64) -> Result<Vec<FamilyMemberDetail>, String>
```

---

## Frontend: Svelte Komponenten

### Bestehende Komponenten (zu erweitern)

- **`PersonForm.svelte`**: Neue Felder hinzufügen (first_name, last_name getrennt, etc.)
- **`Graph.svelte`**: Familienbeziehungen sichtbar machen (optional: andere Farbe/Stil für family-edges)

### Neue Komponenten

- **`FamilyPanel.svelte`**: Zeigt Familie der selektierten Person, ermöglicht Hinzufügen/Entfernen
  - Dropdown für Beziehungstyp (Ehepartner, Kind, Mutter, Vater, Geschwister, etc.)
  - Suchfeld zum Verlinken mit bestehenden Personen oder neue Person erstellen
  - Liste der aktuellen Familienmitglieder mit Bearbeitungsoptionen

### API-Layer (`src/lib/api.js`)

```javascript
export const addPerson = (payload) => invoke('add_person', { payload })
export const updatePerson = (payload) => invoke('update_person', { payload })
export const getPerson = (id) => invoke('get_person', { id })
export const addFamilyMember = (personId, familyId, relationType) => invoke('add_family_member', { person_id: personId, family_id: familyId, relation_type: relationType })
export const removeFamilyMember = (personId, familyId) => invoke('remove_family_member', { person_id: personId, family_id: familyId })
export const getFamily = (personId) => invoke('get_family', { person_id: personId })
```

---

## Workflow beim Hinzufügen einer Familienbeziehung

1. **Nutzer wählt Person A** → `FamilyPanel.svelte` lädt Familie
2. **Nutzer gibt Beziehungstyp (z.B. "Ehepartner") + Name ein**
3. **System sucht nach Person mit diesem Namen**:
   - **Gefunden**: Person B wird verlinkt, Relationship erstellt
   - **Nicht gefunden**: Neue Person B mit last_name = Name wird erstellt, verlinkt, Relationship erstellt
4. **`getGraph()` wird aufgerufen** → Graph wird mit neuer Beziehung aktualisiert

---

## Offene Fragen / Zukünftige Erweiterungen

- Sollen Familienbeziehungen im Graph mit anderen Farben/Stilen gekennzeichnet werden?
- Soll es möglich sein, Familienbeziehungen bidirektional zu machen (z.B. "Mutter" ↔ "Kind")?
- UI für Massenedit von Personen (z.B. mehrere Familienmitglieder auf einmal)?
- Export/Import von Personendaten?

---

## Implementierungs-Roadmap

1. **Database Migration** → `0002_extended_people.sql` + Daten migrieren
2. **Rust Backend** → Models + Commands + DB-Funktionen
3. **API-Layer** → `src/lib/api.js` erweitern
4. **Svelte Komponenten** → `PersonForm.svelte` + neue `FamilyPanel.svelte`
5. **Integration** → `App.svelte` mit FamilyPanel verbinden
6. **Testing** → E2E Tests für Familie hinzufügen/entfernen
