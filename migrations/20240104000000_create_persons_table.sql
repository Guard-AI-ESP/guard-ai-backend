-- Table des personnes connues du système de reconnaissance faciale.
-- L'embedding (vecteur FaceNet 512-dim) est stocké en JSON pour rester
-- portable sans extension vectorielle SQLite.

CREATE TABLE IF NOT EXISTS persons (
    id          TEXT    PRIMARY KEY NOT NULL,   -- UUID v4
    name        TEXT    NOT NULL,
    embedding   TEXT    NOT NULL,               -- JSON array of 512 floats
    photo_url   TEXT,                           -- URL ou chemin local (optionnel)
    created_at  TEXT    NOT NULL                -- ISO-8601 UTC
);

CREATE INDEX IF NOT EXISTS idx_persons_name ON persons(name);
