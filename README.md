# Pi Kiosk

Local-first Rust + Leptos management platform for Raspberry Pi 4B (8 GB) with DSI touchscreen, CSI camera, PoE, and SIM7600G-H cellular HAT.

Also supports:
- Raspberry Pi OS Lite (64-bit and 32-bit)
- Raspberry Pi Desktop (32-bit, including KVM/QEMU x86_64 testing)
- Debian 12
- Ubuntu 24.04
- Native x86_64 development builds

Combines:
- **OpenWrt-like** local network management (LAN, DHCP, DNS, firewall, Wi-Fi modes)
- **Tapo-style** security camera management (live preview, recording, detection)
- **Local AI detection** — motion pre-filter → person detection (tract-onnx / YOLO-nano)
- **SIM7600G-H cellular modem** management (LTE/4G, SMS, AT commands)
- **Wi-Fi hotspot / repeater / VPN / failover** controls

Runs as a kiosk on the DSI touchscreen and is also accessible from LAN devices.

## Quick Start

```bash
# Native build (x86_64, arm64, or armhf)
cargo build --release

# Run web server (dev)
cargo run -p pi-kiosk-web
# → http://127.0.0.1:3000

# Run privileged helper (production: as root via systemd)
sudo cargo run -p pi-kiosk-priv
```

## Installation

### On a target device (Pi, Debian, Ubuntu, or VM)

```bash
# Extract a pre-built archive
tar xzf pi-kiosk-<arch>.tar.gz
cd pi-kiosk-<arch>

# Install as root
sudo ./install.sh

# Start services
sudo systemctl start pi-kiosk-priv pi-kiosk-web
```

### From source (build + install as root)

```bash
# Installs to /usr/local by default. Build runs as the invoking user, not root.
sudo ./scripts/install.sh

# Or install without building
sudo ./scripts/install.sh --no-build

# Build for a specific target
sudo ./scripts/install.sh --target x86_64-unknown-linux-gnu
sudo ./scripts/install.sh --target aarch64-unknown-linux-gnu
sudo ./scripts/install.sh --target armv7-unknown-linux-gnueabihf
```

## Running in a VM or on Debian/Ubuntu x86 (no Pi hardware)

If you are running Raspberry Pi Desktop (x86), Debian, or Ubuntu inside a
KVM/QEMU VM — or on any x86_64 machine — you do **not** need an ARM disk
image or `build-image.sh`. Build natively and install directly:

```bash
# 1. Build (as your normal user, not root)
cargo build --release

# 2. Install the compiled binaries + systemd units
sudo ./scripts/install.sh --no-build

# 3. Start the services
sudo systemctl start pi-kiosk-priv pi-kiosk-web

# 4. Open the dashboard
xdg-open http://localhost:3000
```

The install script detects non-Pi hardware automatically and skips all
Pi-specific steps (boot `config.txt` merge, `gpio`/`i2c` groups). The udev
modem rule is only installed if `udev/99-pi-kiosk-modem.rules` is present
and is harmless in a VM.

Alternatively, run straight from the build tree without installing at all:

```bash
# Terminal 1 — privileged helper (needs root for network/hardware ops)
sudo ./target/release/pi-kiosk-priv

# Terminal 2 — web server
./target/release/pi-kiosk-web
# → http://127.0.0.1:3000
```

## Cross-Compilation

Build on x86_64 for any supported target:

```bash
# Native x86_64 (for testing, KVM/QEMU, or development)
./scripts/cross-compile.sh x86_64

# 64-bit ARM (aarch64) — recommended for Pi 4B 8GB
./scripts/cross-compile.sh arm64

# 32-bit ARM (armhf) — for Pi Desktop OS 32-bit
./scripts/cross-compile.sh armhf
```

For `arm64` / `armhf` on Debian/Ubuntu hosts:

```bash
# 64-bit ARM
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu
./scripts/cross-compile.sh arm64

# 32-bit ARM
rustup target add armv7-unknown-linux-gnueabihf
sudo apt install gcc-arm-linux-gnueabihf
./scripts/cross-compile.sh armhf
```

On hosts without a matching cross toolchain, `cross-compile.sh` uses the `cross` tool with podman/docker containers.

## Building an SD Card Image (Raspberry Pi OS)

> **Note:** `build-image.sh` is only for producing bootable SD card images
> to flash onto physical Raspberry Pi hardware. It requires a base
> Raspberry Pi OS Lite image (e.g. `build/raspios-bookworm-arm64-lite.img`).
> It is **not required** for running pi-kiosk in a VM or on Debian/Ubuntu —
> see [Running in a VM](#running-in-a-vm-or-on-debianubuntu-x86-no-pi-hardware) above.

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
├── Cross.toml              # `cross` tool container config
├── .cargo/config.toml      # cross-compile linker config
├── ARCHITECTURE.md         # full architecture & design document
├── migrations/             # SQLite schema
├── systemd/                # systemd unit files
├── udev/                   # udev rules for modem, GPS, audio, radio
├── config/                 # boot config.txt fragments (DT overlays)
├── scripts/
│   ├── build-image.sh      # SD image builder (--arch arm64|armhf)
│   ├── cross-compile.sh    # cross-compile / native build helper
│   ├── install.sh          # build+install or install-only
│   └── kiosk-browser-wrapper.sh  # auto-detects chromium/chromium-browser
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

## Supported Environments

| Environment                       | Architecture | Notes                                         |
|-----------------------------------|--------------|-----------------------------------------------|
| Raspberry Pi OS Lite 64-bit       | aarch64      | Primary target                                |
| Raspberry Pi OS Lite 32-bit       | armv7l       | `--arch armhf`                                |
| Raspberry Pi Desktop 32-bit       | armv7l       | Also works in KVM/QEMU on x86_64 hosts        |
| Debian 12                         | x86_64/arm64 | No Pi-specific hardware needed for testing    |
| Ubuntu 24.04                      | x86_64/arm64 | No Pi-specific hardware needed for testing    |
| Native x86_64 development         | x86_64       | `cargo build --release` or `cross-compile.sh x86_64` |

All hardware components are optional — the system degrades gracefully when a HAT is absent.

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
