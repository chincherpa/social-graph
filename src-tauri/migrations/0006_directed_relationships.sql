-- Beziehungen sind jetzt gerichtet: from_id ist [kind] von to_id.
-- Bisher zwang CHECK(from_id < to_id) Sortierung nach ID -> Richtung ging verloren
-- (z.B. "Tochter" liess sich nicht zuordnen, wer wessen Tochter ist).
-- Uniqueness pro Personenpaar wird jetzt in Rust geprueft statt per DB-Constraint,
-- da SQLite kein UNIQUE auf einem ungeordneten Paar ausdruecken kann.
CREATE TABLE relationships_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    to_id       INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL DEFAULT 'kennt',
    strength    INTEGER NOT NULL DEFAULT 3 CHECK (strength BETWEEN 1 AND 5),
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (from_id != to_id)
);

INSERT INTO relationships_new (id, from_id, to_id, kind, strength, note, created_at)
SELECT id, from_id, to_id, kind, strength, note, created_at FROM relationships;

DROP TABLE relationships;
ALTER TABLE relationships_new RENAME TO relationships;

CREATE INDEX IF NOT EXISTS idx_rel_from ON relationships(from_id);
CREATE INDEX IF NOT EXISTS idx_rel_to   ON relationships(to_id);
