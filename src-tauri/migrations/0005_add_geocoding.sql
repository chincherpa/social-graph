-- Geokoordinaten für Weltkarten-Ansicht
ALTER TABLE people ADD COLUMN lat REAL;
ALTER TABLE people ADD COLUMN lon REAL;
ALTER TABLE people ADD COLUMN geocoded_at TEXT;
