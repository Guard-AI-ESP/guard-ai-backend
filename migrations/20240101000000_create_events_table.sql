-- Migration: Create events table for Guard-AI event persistence

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Identifiants métier
    event_id TEXT NOT NULL UNIQUE,
    site_id TEXT NOT NULL,
    hub_id TEXT,

    -- Classification de l'événement
    source TEXT NOT NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,

    -- Temporalité
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Données
    payload TEXT NOT NULL,
    media_ref TEXT,
    tags TEXT NOT NULL DEFAULT '[]',

    -- Métadonnées
    schema_version TEXT NOT NULL
);

-- Index pour les filtres fréquents
CREATE INDEX IF NOT EXISTS idx_events_site_id ON events(site_id);
CREATE INDEX IF NOT EXISTS idx_events_source ON events(source);
CREATE INDEX IF NOT EXISTS idx_events_severity ON events(severity);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_site_timestamp ON events(site_id, timestamp);
