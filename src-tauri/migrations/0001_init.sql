-- Personen im sozialen Graphen
CREATE TABLE IF NOT EXISTS people (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    note        TEXT,
    color       TEXT NOT NULL DEFAULT '#3b82f6',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Beziehungen zwischen zwei Personen (ungerichtet).
-- Konvention: from_id ist immer der kleinere id-Wert, to_id der größere.
-- Das verhindert doppelte Kanten (A-B und B-A) und macht UNIQUE eindeutig.
CREATE TABLE IF NOT EXISTS relationships (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id     INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    to_id       INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL DEFAULT 'kennt',
    strength    INTEGER NOT NULL DEFAULT 3 CHECK (strength BETWEEN 1 AND 5),
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (from_id < to_id),
    UNIQUE (from_id, to_id)
);

CREATE INDEX IF NOT EXISTS idx_rel_from ON relationships(from_id);
CREATE INDEX IF NOT EXISTS idx_rel_to   ON relationships(to_id);
