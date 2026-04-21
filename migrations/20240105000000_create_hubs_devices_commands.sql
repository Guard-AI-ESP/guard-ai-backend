-- Tables pour l'intégration cyber (Raspberry Pi + hostapd + dnsmasq + suricata + iptables).
-- hubs     : registre des hubs réseau provisionnés (un par site)
-- devices  : snapshot des clients actuellement connectés, poussé par le hub
-- commands : queue des commandes (scan_network, block_device, kick_device, unblock_device)
--            envoyées du backend au hub via WebSocket

CREATE TABLE IF NOT EXISTS hubs (
    id              TEXT    PRIMARY KEY NOT NULL,       -- identifiant stable, ex. "pi-hub-01"
    site_id         TEXT    NOT NULL,                   -- UUID v4 du site
    name            TEXT    NOT NULL,
    api_key_hash    TEXT    NOT NULL,                   -- bcrypt de la clé M2M (jamais stockée en clair)
    last_seen_at    TEXT,                               -- ISO-8601 UTC, mis à jour par heartbeat
    created_at      TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_hubs_site_id ON hubs(site_id);

CREATE TABLE IF NOT EXISTS devices (
    mac_address     TEXT    PRIMARY KEY NOT NULL,       -- format aa:bb:cc:11:22:33 (lower, strict)
    hub_id          TEXT    NOT NULL REFERENCES hubs(id) ON DELETE CASCADE,
    ip_address      TEXT,                               -- IPv4, mis à jour via DHCP
    hostname        TEXT,
    rssi            INTEGER,                            -- force signal Wi-Fi (-120..0)
    first_seen_at   TEXT    NOT NULL,
    last_seen_at    TEXT    NOT NULL,                   -- rafraîchi à chaque snapshot PUT /v1/devices
    connected       INTEGER NOT NULL DEFAULT 1          -- 0/1 plutôt que bool pour portabilité SQLite
);

CREATE INDEX IF NOT EXISTS idx_devices_hub_id ON devices(hub_id);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen ON devices(last_seen_at);

CREATE TABLE IF NOT EXISTS commands (
    id              TEXT    PRIMARY KEY NOT NULL,       -- UUID v4
    hub_id          TEXT    NOT NULL REFERENCES hubs(id) ON DELETE CASCADE,
    type            TEXT    NOT NULL,                   -- scan_network | block_device | kick_device | unblock_device
    payload         TEXT    NOT NULL,                   -- JSON (ex: {"mac_address":"..."})
    status          TEXT    NOT NULL DEFAULT 'pending', -- pending|dispatched|running|succeeded|failed
    result          TEXT,                               -- JSON, rempli au retour
    error           TEXT,
    created_at      TEXT    NOT NULL,
    dispatched_at   TEXT,
    completed_at    TEXT,
    expires_at      TEXT                                -- TTL facultatif (auto-unblock iptables)
);

CREATE INDEX IF NOT EXISTS idx_commands_hub_id ON commands(hub_id);
CREATE INDEX IF NOT EXISTS idx_commands_status ON commands(status);
CREATE INDEX IF NOT EXISTS idx_commands_created_at ON commands(created_at);
