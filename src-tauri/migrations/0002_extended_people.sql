-- Erweiterte Personendaten + Familienbeziehungen
ALTER TABLE people ADD COLUMN first_name TEXT;
ALTER TABLE people ADD COLUMN last_name TEXT NOT NULL DEFAULT '';
ALTER TABLE people ADD COLUMN nickname TEXT;
ALTER TABLE people ADD COLUMN birth_date TEXT;
ALTER TABLE people ADD COLUMN known_since TEXT;
ALTER TABLE people ADD COLUMN address TEXT;
ALTER TABLE people ADD COLUMN employer TEXT;

UPDATE people SET last_name = name;

ALTER TABLE people DROP COLUMN name;

-- Familienmitglieder: Verlinkt Personen als Familie
CREATE TABLE IF NOT EXISTS family_members (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    family_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(person_id, family_id),
    CHECK(person_id != family_id)
);

CREATE INDEX IF NOT EXISTS idx_family_person ON family_members(person_id);
CREATE INDEX IF NOT EXISTS idx_family_member ON family_members(family_id);
