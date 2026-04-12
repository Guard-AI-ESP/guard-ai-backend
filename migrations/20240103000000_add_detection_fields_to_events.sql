-- Migration: Add face detection fields to events table
--
-- Ces champs sont NULL pour les events non-caméra (network, sensor, system).
-- Ils sont remplis par le pipeline ML quand source = 'camera'.

ALTER TABLE events ADD COLUMN camera_id TEXT;
ALTER TABLE events ADD COLUMN face_id TEXT;
ALTER TABLE events ADD COLUMN person_name TEXT;
ALTER TABLE events ADD COLUMN confidence REAL;
ALTER TABLE events ADD COLUMN is_known INTEGER;  -- 0 = inconnu, 1 = connu (SQLite n'a pas de BOOLEAN natif)
ALTER TABLE events ADD COLUMN bounding_box TEXT; -- JSON { "x": 0.1, "y": 0.2, "w": 0.3, "h": 0.4 } coords normalisées

-- Index pour les requêtes fréquentes sur les détections
CREATE INDEX IF NOT EXISTS idx_events_camera_id ON events(camera_id);
CREATE INDEX IF NOT EXISTS idx_events_face_id ON events(face_id);
CREATE INDEX IF NOT EXISTS idx_events_is_known ON events(is_known);
