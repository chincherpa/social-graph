-- Letzter Kontakt: Art und Datum
ALTER TABLE people ADD COLUMN last_contact_type TEXT;
ALTER TABLE people ADD COLUMN last_contact_date TEXT;
