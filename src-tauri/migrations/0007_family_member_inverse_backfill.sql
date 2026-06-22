-- Bisher wurde beim Anlegen eines Familienmitglieds nur eine Richtung gespeichert
-- (A ist Kind von B), die Gegenseite (B ist Mutter/Vater von A) fehlte komplett.
-- Backfill für bereits bestehende Datensätze, geschlechtsabhängig bei "Kind".
INSERT INTO family_members (person_id, family_id, relation_type)
SELECT fm.family_id, fm.person_id,
  CASE fm.relation_type
    WHEN 'Kind' THEN CASE (SELECT gender FROM people WHERE id = fm.family_id)
      WHEN 'w' THEN 'Mutter'
      WHEN 'm' THEN 'Vater'
      ELSE 'Elternteil'
    END
    WHEN 'Mutter' THEN 'Kind'
    WHEN 'Vater' THEN 'Kind'
    WHEN 'Ehepartner' THEN 'Ehepartner'
    WHEN 'Geschwister' THEN 'Geschwister'
    ELSE 'Sonstige'
  END
FROM family_members fm
WHERE NOT EXISTS (
  SELECT 1 FROM family_members fm2
  WHERE fm2.person_id = fm.family_id AND fm2.family_id = fm.person_id
);
