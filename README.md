# Pi Kiosk

Local-first Rust + Leptos management platform for Raspberry Pi 4B (8 GB) with DSI touchscreen, CSI camera, PoE, and SIM7600G-H cellular HAT.

Combines:
- **OpenWrt-like** local network management (LAN, DHCP, DNS, firewall, Wi-Fi modes)
- **Tapo-style** security camera management (live preview, recording, detection)
- **Local AI detection** — motion pre-filter → person detection (tract-onnx / YOLO-nano)
- **SIM7600G-H cellular modem** management (LTE/4G, SMS, AT commands)
- **Wi-Fi hotspot / repeater / VPN / failover** controls

Runs as a kiosk on the DSI touchscreen and is also accessible from LAN devices.

## Quick Start

```bash
# Apply DB migration
sqlite3 /var/lib/pi-kiosk/kiosk.db < migrations/001_init.sql

# Build all crates (native, on Pi or matching arch)
cargo build --release

# Run web server (dev)
cargo run -p pi-kiosk-web
# → http://127.0.0.1:3000

# Run privileged helper (production: as root via systemd)
sudo cargo run -p pi-kiosk-priv
```

## Cross-Compilation

Build on x86_64 for Raspberry Pi (32-bit or 64-bit):

```bash
# 64-bit (aarch64) — recommended for Pi 4B 8GB
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu
./scripts/cross-compile.sh arm64

# 32-bit (armhf) — for Pi Desktop OS 32-bit
rustup target add armv7-unknown-linux-gnueabihf
sudo apt install gcc-arm-linux-gnueabihf
./scripts/cross-compile.sh armhf
```

## Building an SD Card Image

```bash
# 64-bit (default)
sudo bash scripts/build-image.sh --arch arm64

# 32-bit
sudo bash scripts/build-image.sh --arch armhf

# Stage files only (no chroot, useful for inspection)
sudo bash scripts/build-image.sh --arch armhf --stage-only
```

Output: `images/pi-kiosk.img` — flash with `dd` or Raspberry Pi Imager.

## Repository Layout

```
pi-kiosk/
├── Cargo.toml              # workspace root
├── .cargo/config.toml      # cross-compile linker config
├── ARCHITECTURE.md         # full architecture & design document
├── migrations/             # SQLite schema
├── systemd/                # systemd unit files
├── udev/                   # udev rules for modem, GPS, audio, radio
├── config/                 # boot config.txt fragments (DT overlays)
├── scripts/
│   ├── build-image.sh      # SD image builder (--arch arm64|armhf)
│   ├── cross-compile.sh    # cross-compile helper
│   └── install.sh          # direct install on existing Pi
└── crates/
    ├── core/               # shared types, config, hardware detection
    ├── audio/              # ALSA audio device detection + monitoring
    ├── camera/             # camera capture, MJPEG, clips
    ├── db/                 # SQLite database layer (rusqlite, WAL)
    ├── detection/          # motion + person detection (tract-onnx)
    ├── gps/                # NMEA parsing + GPS serial reader
    ├── modem/              # SIM7600 AT commands + ModemManager
    ├── network/            # network status, WAN monitoring
    ├── privileged/         # root helper (hostapd, nftables, VPN, cellular)
    ├── radio/              # LoRa/mesh radio serial communication
    └── web/                # Leptos SSR + Axum server, pages, server functions
```

## Phased Roadmap

| Phase | Description                              | Status |
|-------|------------------------------------------|--------|
| 1     | Architecture & repo skeleton             | ✅      |
| 2     | Leptos touchscreen dashboard UI          | Next   |
| 3     | Backend service layer & state management |        |
| 4     | Camera preview & local capture           |        |
| 5     | Motion detection & event logging         |        |
| 6     | Person detection (tract-onnx)            |        |
| 7     | Network status dashboard                 |        |
| 8     | hostapd / dnsmasq / nftables helpers     |        |
| 9     | SIM7600G-H modem detection & status      |        |
| 10    | Cellular failover logic                  |        |
| 11    | VPN controls                             |        |
| 12    | Notifications & alert rules              |        |
| 13    | Hardening, systemd, deployment           |        |

See `ARCHITECTURE.md` for full design details.

## Hardware Target

- Raspberry Pi 4B 8GB
- Raspberry Pi DSI touchscreen (800×480)
- CSI ribbon camera with night-vision
- PoE power
- SIM7600G-H HAT (LTE/4G + SMS)
- Built-in Wi-Fi 2.4/5 GHz
- Optional external USB Wi-Fi adapter for concurrent AP+STA

## License

MIT
