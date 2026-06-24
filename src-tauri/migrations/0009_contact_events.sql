-- Kontakt-Verlauf: jeder Kontakt = eine Zeile
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
