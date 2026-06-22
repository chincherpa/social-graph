-- Profilbild-Unterstützung: Dateipfad + Kreis-Ausschnitt-Koordinaten
ALTER TABLE people ADD COLUMN image_path TEXT;
ALTER TABLE people ADD COLUMN image_crop_x INTEGER;
ALTER TABLE people ADD COLUMN image_crop_y INTEGER;
ALTER TABLE people ADD COLUMN image_crop_radius INTEGER;
