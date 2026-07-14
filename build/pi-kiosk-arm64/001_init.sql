-- pi-kiosk initial schema
-- SQLite, WAL mode

PRAGMA foreign_keys = ON;

-- ── Settings (key-value, JSON values) ──────────────────────────
CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Network profiles ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS network_profiles (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    mode        TEXT NOT NULL CHECK (mode IN ('ap','client','repeater','hotspot')),
    ssid        TEXT NOT NULL,
    password    TEXT,
    band        TEXT NOT NULL DEFAULT '2.4g' CHECK (band IN ('2.4g','5g')),
    channel     INTEGER,
    encryption  TEXT NOT NULL DEFAULT 'wpa2_psk' CHECK (encryption IN ('open','wpa2_psk','wpa3_sae')),
    country_code TEXT DEFAULT 'US',
    interface   TEXT,
    is_active   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── DHCP leases (snapshot/cache) ───────────────────────────────
CREATE TABLE IF NOT EXISTS dhcp_leases (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    mac             TEXT NOT NULL,
    ip              TEXT NOT NULL,
    hostname        TEXT,
    interface       TEXT,
    lease_expires   TEXT,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(mac, ip)
);

-- ── Firewall rules (port forwarding + nftables metadata) ───────
CREATE TABLE IF NOT EXISTS firewall_rules (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    proto       TEXT NOT NULL CHECK (proto IN ('tcp','udp','any')),
    src_port    TEXT,
    dest_ip     TEXT NOT NULL,
    dest_port   TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── WAN sources / failover priority ────────────────────────────
CREATE TABLE IF NOT EXISTS wan_sources (
    id              TEXT PRIMARY KEY,
    source          TEXT NOT NULL CHECK (source IN ('ethernet','wifi','cellular')),
    priority        INTEGER NOT NULL,
    health_check_target TEXT,
    health_check_type   TEXT DEFAULT 'ping' CHECK (health_check_type IN ('ping','dns','http')),
    enabled         INTEGER NOT NULL DEFAULT 1
);

-- ── Failover log ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS failover_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    from_source TEXT,
    to_source   TEXT NOT NULL,
    reason      TEXT NOT NULL
);

-- ── Camera clips ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS camera_clips (
    id          TEXT PRIMARY KEY,
    file_path   TEXT NOT NULL,
    duration_s  REAL NOT NULL,
    trigger     TEXT CHECK (trigger IN ('manual','motion','person','tamper','scheduled')),
    started_at  TEXT NOT NULL,
    ended_at    TEXT,
    size_bytes  INTEGER,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_clips_started ON camera_clips(started_at);

-- ── Detection events ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS detection_events (
    id              TEXT PRIMARY KEY,
    kind            TEXT NOT NULL CHECK (kind IN ('motion','person','tamper')),
    timestamp       TEXT NOT NULL,
    confidence      REAL,
    thumbnail_path  TEXT,
    metadata        TEXT DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_events_ts ON detection_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_events_kind ON detection_events(kind);

-- ── Modem status (latest snapshot) ─────────────────────────────
CREATE TABLE IF NOT EXISTS modem_status (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    present         INTEGER NOT NULL DEFAULT 0,
    sim_ready       INTEGER NOT NULL DEFAULT 0,
    operator        TEXT,
    signal_strength INTEGER,
    access_technology TEXT,
    registered      INTEGER NOT NULL DEFAULT 0,
    connected       INTEGER NOT NULL DEFAULT 0,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO modem_status (id) VALUES (1);

-- ── SMS messages ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sms_messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    direction   TEXT NOT NULL CHECK (direction IN ('sent','received')),
    number      TEXT NOT NULL,
    body        TEXT NOT NULL,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    status      TEXT DEFAULT 'pending'
);

-- ── VPN configs ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS vpn_configs (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    type        TEXT NOT NULL CHECK (type IN ('wireguard','openvpn')),
    mode        TEXT NOT NULL CHECK (mode IN ('client','server')),
    config_path TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 0,
    kill_switch INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Alert rules ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS alert_rules (
    id              TEXT PRIMARY KEY,
    event_type      TEXT NOT NULL,
    action          TEXT NOT NULL CHECK (action IN ('ui','sms','webhook')),
    action_target   TEXT,
    cooldown_s      INTEGER NOT NULL DEFAULT 300,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Alert log (dispatched notifications) ───────────────────────
CREATE TABLE IF NOT EXISTS alert_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id     TEXT,
    event_type  TEXT NOT NULL,
    action      TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'sent',
    message     TEXT,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE SET NULL
);

-- ── Auth sessions ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS auth_sessions (
    token       TEXT PRIMARY KEY,
    username    TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_expires ON auth_sessions(expires_at);

-- ── Audit log ──────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    action      TEXT NOT NULL,
    user        TEXT,
    details     TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_log(timestamp DESC);

-- ── Users ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'admin' CHECK (role IN ('admin','viewer')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
