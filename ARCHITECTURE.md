# Pi Kiosk — Architecture & Design

Local-first Rust + Leptos management platform for Raspberry Pi 4B (8 GB)
with DSI touchscreen, CSI camera, PoE, and SIM7600G-H cellular HAT.

---

## 1. System Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Raspberry Pi 4B (8 GB)                     │
│                                                              │
│  ┌─────────────┐   ┌──────────────┐   ┌───────────────────┐  │
│  │  DSI Touch  │   │  CSI Camera  │   │  SIM7600G-H HAT   │  │
│  │  (kiosk UI) │   │  (night vis) │   │  (LTE/4G + SMS)   │  │
│  └──────┬──────┘   └──────┬───────┘   └────────┬──────────┘  │
│         │                 │                    │             │
│  ┌──────┴─────────────────┴────────────────────┴──────────┐  │
│  │              pi-kiosk-web  (Leptos + Axum)              │  │
│  │   ┌──────────┐ ┌──────────┐ ┌────────┐ ┌────────────┐  │  │
│  │   │ Dashboard │ │ Camera   │ │ Network│ │  Modem     │  │  │
│  │   │   UI      │ │ Module   │ │ Module │ │  Module    │  │  │
│  │   └─────┬─────┘ └────┬─────┘ └───┬────┘ └─────┬──────┘  │  │
│  │         │            │           │            │         │  │
│  │   ┌─────┴────────────┴───────────┴────────────┴──────┐  │  │
│  │   │            Server Functions (Axum routes)        │  │  │
│  │   └─────────────────────┬───────────────────────────┘  │  │
│  └─────────────────────────┼──────────────────────────────┘  │
│                            │                                  │
│  ┌─────────────────────────┴──────────────────────────────┐  │
│  │              Tokio Background Workers                   │  │
│  │  ┌────────┐ ┌──────────┐ ┌────────┐ ┌───────────────┐  │  │
│  │  │ Camera │ │ Detection│ │Failover│ │ Modem Monitor │  │  │
│  │  │ Capture│ │ Pipeline │ │ Engine │ │ + SMS         │  │  │
│  │  └────────┘ └──────────┘ └────────┘ └───────────────┘  │  │
│  └─────────────────────────┬──────────────────────────────┘  │
│                            │                                  │
│  ┌─────────────────────────┴──────────────────────────────┐  │
│  │           pi-kiosk-privileged  (root helper)            │  │
│  │   hostapd │ dnsmasq │ nftables │ wg-quick │ ifupdown   │  │
│  └────────────────────────────────────────────────────────┘  │
│                            │                                  │
│  ┌─────────────────────────┴──────────────────────────────┐  │
│  │              SQLite  (settings, events, clips)          │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────┐ ┌──────────┐ ┌────────┐ ┌──────────┐ ┌─────────┐  │
│  │PoE   │ │Built-in  │ │USB Wi-Fi│ │Ethernet │ │  GPIO   │  │
│  │Power │ │Wi-Fi 2.4/│ │(optional)│ │(PoE WAN)│ │ (IR LED)│  │
│  │      │ │5 GHz     │ │         │ │         │ │         │  │
│  └──────┘ └──────────┘ └────────┘ └──────────┘ └─────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Process model

| Process           | User        | Responsibility                                   |
|-------------------|-------------|--------------------------------------------------|
| `pi-kiosk-web`    | `pikiosk`   | Leptos SSR + Axum server, background workers     |
| `pi-kiosk-priv`   | `root`      | Privileged helper: network/firewall/hostapd ops  |
| `kiosk-browser`   | `pikiosk`   | Chromium/Firefox kiosk displaying localhost       |

The web process runs as unprivileged user `pikiosk`. All system-level
mutations (interface config, nftables, hostapd, dnsmasq, WireGuard) go
through the privileged helper over a Unix domain socket with a
capability-checked command protocol. The web UI **never** shells out
directly.

### Communication

- **Web ↔ Workers**: in-process Tokio channels + shared `Arc<RwLock<AppState>>`.
- **Web ↔ Privileged helper**: Unix socket, length-prefixed JSON, request
  validation on both sides.
- **Workers ↔ Linux services**: `dbus`/`zbus` (NetworkManager, ModemManager),
  subprocess calls to `rpicam-still`/`rpicam-vid`, file I/O for configs.
- **Frontend ↔ Backend**: Leptos server functions (compiled to Axum routes)
  + SSE/WebSocket for live updates (camera frames, detection events, stats).

---

## 2. Rust Crate / Module Structure

```
pi-kiosk/
├── Cargo.toml                    # workspace root
├── ARCHITECTURE.md
├── README.md
├── migrations/
│   └── 001_init.sql
├── systemd/
│   ├── pi-kiosk-web.service
│   ├── pi-kiosk-priv.service
│   └── kiosk-browser.service
├── crates/
│   ├── core/                     # shared types, config, errors
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs         # AppConfig (toml), hot-reloadable
│   │       ├── error.rs          # unified Error type
│   │       └── types.rs          # WanSource, DetectionEvent, etc.
│   │
│   ├── db/                       # SQLite via rusqlite / sqlx
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs         # typed table definitions
│   │       ├── settings.rs       # key-value + structured settings repo
│   │       ├── events.rs         # detection/network/modem event log
│   │       ├── clips.rs          # camera clip metadata + retention
│   │       └── network.rs        # profiles, leases, firewall rules
│   │
│   ├── privileged/               # root helper binary + protocol
│   │   └── src/
│   │       ├── main.rs           # listens on Unix socket, dispatches
│   │       ├── proto.rs          # Request/Response enum (shared)
│   │       ├── hostapd.rs        # generate config, start/stop service
│   │       ├── dnsmasq.rs        # DHCP/DNS config
│   │       ├── nftables.rs       # firewall rules
│   │       ├── networkd.rs       # systemd-networkd / NM profiles
│   │       └── wireguard.rs      # wg-quick up/down
│   │
│   ├── camera/                   # CSI camera via rpicam/libcamera
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── capture.rs        # rpicam-still/vid subprocess + MJPEG
│   │       ├── nightvision.rs    # IR mode, exposure, auto-darkness detect
│   │       └── storage.rs        # clip writer, retention enforcement
│   │
│   ├── detection/                # motion + person detection pipeline
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── motion.rs         # frame-difference motion pre-filter
│   │       ├── person.rs         # tract-onnx YOLO-nano inference
│   │       ├── tamper.rs         # darkness/obstruction/movement detect
│   │       └── pipeline.rs       # orchestrator: capture→motion→person
│   │
│   ├── network/                  # network status + control (unprivileged)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── status.rs         # read interfaces, links, routes
│   │       ├── clients.rs        # connected Wi-Fi clients, DHCP leases
│   │       ├── wifi.rs           # AP/client/repeater/hotspot logic
│   │       └── priv_client.rs    # talks to privileged helper
│   │
│   ├── modem/                    # SIM7600G-H management
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mmcli.rs          # ModemManager via zbus/mmcli
│   │       ├── at.rs             # raw AT command fallback over serial
│   │       ├── sms.rs            # send/receive SMS
│   │       └── diagnostics.rs    # signal, tech, operator, SIM status
│   │
│   ├── failover/                 # WAN failover engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── health.rs         # ping/DNS/HTTP health checks
│   │       └── engine.rs         # priority-based switching + logging
│   │
│   ├── vpn/                      # WireGuard / OpenVPN
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── wireguard.rs      # config gen, status via privileged helper
│   │       └── openvpn.rs        # optional
│   │
│   ├── notify/                   # notification + alert rules
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rules.rs          # event→notification rule engine
│   │       ├── sms.rs            # via modem crate
│   │       ├── webhook.rs        # HTTP POST alerts
│   │       └── ui.rs             # in-app notification queue (SSE)
│   │
│   └── web/                      # Leptos + Axum application
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs           # Axum server + worker spawn
│       │   ├── app.rs            # Leptos root component + router
│       │   ├── state.rs          # AppState (shared with workers)
│       │   ├── sse.rs            # Server-Sent Events endpoint
│       │   ├── auth.rs           # local session auth
│       │   ├── server_fns/
│       │   │   ├── dashboard.rs
│       │   │   ├── camera.rs
│       │   │   ├── network.rs
│       │   │   ├── modem.rs
│       │   │   ├── failover.rs
│       │   │   ├── vpn.rs
│       │   │   └── settings.rs
│       │   └── pages/
│       │       ├── dashboard.rs
│       │       ├── camera.rs
│       │       ├── detection.rs
│       │       ├── network.rs
│       │       ├── modem.rs
│       │       ├── failover.rs
│       │       ├── vpn.rs
│       │       ├── notifications.rs
│       │       └── settings.rs
│       ├── style/
│       │   └── main.css
│       └── assets/
│           └── tailwind.config.js
```

### Dependency graph (simplified)

```
web ──► core, db, camera, detection, network, modem, failover, vpn, notify
camera ──► core, db
detection ──► core, camera, db
network ──► core, db, privileged (proto only)
modem ──► core, db
failover ──► core, db, network, modem
vpn ──► core, privileged (proto only)
notify ──► core, db, modem
privileged ──► core
db ──► core
```

---

## 3. Linux Services & Tools Integration

| Component              | Tool / Service              | Integration method                  |
|------------------------|-----------------------------|-------------------------------------|
| Camera capture         | `rpicam-still`, `rpicam-vid`| subprocess + libcamera              |
| Night vision IR        | GPIO pin (IR LED cut filter)| `gpio` sysfs or `rppal` crate       |
| Wi-Fi AP               | `hostapd`                   | config generation + systemd         |
| DHCP / DNS             | `dnsmasq`                   | config generation + systemd         |
| Firewall               | `nftables`                  | `nft` via privileged helper         |
| Interface management   | `systemd-networkd` / NM     | `zbus` or `networkctl`              |
| Modem control          | `ModemManager` + `mmcli`    | `zbus` (D-Bus) + AT serial fallback |
| VPN — WireGuard        | `wireguard-tools` (`wg-quick`)| config gen + privileged helper   |
| VPN — OpenVPN          | `openvpn`                   | config gen + privileged helper      |
| Health checks          | `ping`, DNS queries         | `tokio::process` + `hickory-dns`    |
| Kiosk browser          | `chromium-browser --kiosk`  | systemd user service                |
| Display                | Wayland / X11               | `weston` or X11 for DSI             |
| Time sync              | `systemd-timesyncd` / `chrony`| `timedatectl` via privileged helper|
| Logging                | `tracing` → journald        | `tracing-journald`                  |
| DB                     | SQLite                      | embedded, WAL mode                  |
| Inference              | `tract-onnx`                | Rust crate, YOLO-nano ONNX model    |

### Wi-Fi hardware constraint handling

The Pi 4B's built-in Wi-Fi (CYW43455) supports **concurrent AP + STA only
on the same channel/band**. The software detects this via `iw list` and:

- If only built-in Wi-Fi: restrict AP+STA to same band/channel, or
  operate in AP-only or STA-only mode.
- If external USB Wi-Fi detected: assign one adapter to STA/uplink and
  the other to AP for full repeater/hotspot operation.
- UI shows a recommendation banner when a mode requires an adapter that
  isn't present.

---

## 4. Database Schema (SQLite)

See `migrations/001_init.sql` for full DDL. Summary:

| Table              | Purpose                                              |
|--------------------|------------------------------------------------------|
| `settings`         | key-value app settings (JSON values)                 |
| `network_profiles` | saved Wi-Fi/AP/repeater configurations               |
| `dhcp_leases`      | current/snapshot DHCP lease cache                    |
| `firewall_rules`   | port forwarding + nftables rule metadata             |
| `wan_sources`      | configured WAN priority + health check params        |
| `failover_log`     | failover state-change events                         |
| `camera_clips`     | recorded clip metadata (path, duration, trigger)     |
| `detection_events` | motion/person/tamper events with timestamps + thumbs |
| `modem_status`     | latest modem snapshot (signal, tech, operator)       |
| `sms_messages`     | sent/received SMS log                                |
| `vpn_configs`      | WireGuard/OpenVPN profile metadata                   |
| `alert_rules`      | event-type → notification action mapping             |
| `alert_log`        | dispatched notifications                             |
| `auth_sessions`    | local session tokens                                 |
| `audit_log`        | security-relevant action log                         |

SQLite is opened in **WAL mode** with a single writer connection pooled
behind a `tokio::sync::Mutex`; readers use read-only connections.

---

## 5. Privilege Model

```
┌────────────────┐     Unix socket      ┌──────────────────┐
│  pi-kiosk-web  │ ──────────────────► │ pi-kiosk-priv    │
│  (user:        │   JSON request       │ (user: root)     │
│   pikiosk)     │   validated both     │                  │
│                │   sides              │ - hostapd        │
│  NO root       │ ◄────────────────── │ - dnsmasq        │
│  NO shell      │   JSON response      │ - nftables       │
│  NO raw sockets│                      │ - wg-quick       │
└────────────────┘                      │ - networkctl     │
                                        │ - timedatectl    │
                                        └──────────────────┘
```

### Principles

1. **Web process is unprivileged** — runs as `pikiosk`, no `sudo`, no
   `CAP_NET_ADMIN`, no shell execution of user-controlled strings.
2. **Privileged helper is a fixed binary** — not a generic shell. It
   accepts a typed `PrivRequest` enum, validates every field, and
   executes only predefined operations.
3. **Socket permissions** — `/run/pi-kiosk/priv.sock` owned by
   `root:pikiosk`, mode `0660`. Only `pikiosk` group can connect.
4. **No arbitrary command execution** — the helper never runs
   `system("...")` with user input. All commands are constructed from
   typed structs.
5. **Audit logging** — every privileged operation is logged to
   `audit_log` with timestamp, requesting user, and parameters.
6. **Secrets** — passwords/keys stored in SQLite with OS keyring
   integration where available, or file-based with `0600` permissions.
   VPN private keys written by the privileged helper to
   `/etc/wireguard/` with `0600`.
7. **Network exposure** — Axum binds to `127.0.0.1` and optionally LAN
   IP; never `0.0.0.0` by default. Firewall rules block WAN-side access
   to the management port.
8. **Authentication** — local username/password (argon2 hash), session
   cookie. All server functions require auth except the login endpoint.

---

## 6. Phased MVP Roadmap

### Phase 1 — Architecture & repo skeleton ✅ (this commit)
- Workspace structure, core types, DB schema, privileged helper proto,
  minimal Leptos + Axum app that compiles and serves a dashboard shell.

### Phase 2 — Leptos touchscreen dashboard UI
- Dashboard page with status cards (WAN, camera, CPU/temp/RAM, clients).
- Responsive layout optimized for 800×480 DSI touchscreen.
- SSE live-update channel for stats.
- Navigation to all module pages (stubbed).

### Phase 3 — Backend service layer & state management
- `AppState` with `Arc<RwLock<>>` shared state.
- Background worker framework (tokio tasks with graceful shutdown).
- Settings repository + hot-reload.
- Auth middleware + session management.

### Phase 4 — Camera preview & local capture
- `rpicam-vid` subprocess → MJPEG stream → SSE/WebSocket to frontend.
- Snapshot capture, recording start/stop.
- Night-vision auto-detection (brightness histogram → IR toggle).
- Clip storage with retention enforcement.

### Phase 5 — Motion detection & event logging
- Frame-difference motion detector (configurable threshold/interval).
- Event log UI (timeline).
- Tamper detection (sudden darkness, obstruction, feed loss).

### Phase 6 — Person detection (tract-onnx)
- YOLO-nano ONNX model loaded via `tract-onnx`.
- Inference only on motion-flagged frames.
- Person detection toggle, confidence threshold, cooldown.
- Bounding-box overlay on preview (optional).

### Phase 7 — Network status dashboard
- Read interfaces, links, routes, signal strength.
- Connected clients list, DHCP leases.
- Wi-Fi mode display (AP/STA/repeater).

### Phase 8 — hostapd / dnsmasq / nftables helpers
- Privileged helper operations for AP, DHCP, firewall.
- Wi-Fi AP/STA/repeater/hotspot configuration UI.
- Port forwarding rules.
- External USB Wi-Fi adapter detection + recommendations.

### Phase 9 — SIM7600G-H modem detection & status
- ModemManager D-Bus integration (`zbus`).
- AT command serial fallback.
- SIM status, signal, operator, technology, diagnostics.
- SMS send/receive.

### Phase 10 — Cellular failover logic
- Health-check engine (ping/DNS/HTTP).
- Priority-based WAN switching via privileged helper.
- Automatic return to preferred WAN.
- Failover event log + UI controls.

### Phase 11 — VPN controls
- WireGuard config generation, start/stop, status.
- OpenVPN (optional).
- Kill-switch via nftables.
- Import/export configuration.

### Phase 12 — Notifications & alert rules
- Alert rule engine (event type → action).
- SMS, webhook, in-app UI notifications.
- Cooldown/dedup logic.

### Phase 13 — Hardening, systemd, deployment
- systemd unit files, privilege separation.
- Read-only root filesystem considerations.
- HTTPS (self-signed or Let's Encrypt via LAN CA).
- Backup/restore configuration.
- Image build script (pi-gen / custom SD image).

---

## 7. First Minimal Project Skeleton

The skeleton in this commit provides:

- **Cargo workspace** with `core`, `db`, `privileged`, `web` crates.
- **Leptos + Axum** app serving a dashboard shell with navigation.
- **SQLite migration** with full schema.
- **Privileged helper** binary with typed request protocol.
- **systemd service** templates.
- `cargo check` passes.

### Run the skeleton

```bash
# Apply DB migration
sqlite3 /var/lib/pi-kiosk/kiosk.db < migrations/001_init.sql

# Build
cargo build --release

# Run web server (dev mode)
cargo run -p pi-kiosk-web

# Run privileged helper (as root, in production)
sudo cargo run -p pi-kiosk-priv
```

The web server starts on `http://127.0.0.1:3000`.
