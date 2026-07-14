# Pi-Kiosk — Pre-built Binary Archive

## What's in this archive

- `pi-kiosk-web` — web server + dashboard
- `pi-kiosk-priv` — privileged helper
- `001_init.sql` — SQLite database migration
- `install.sh` — installation script (no Rust toolchain needed)
- `99-pi-kiosk-modem.rules` — udev rules for modem/GPS/audio/radio
- `config.txt` — boot config fragments (DT overlays) for Raspberry Pi
- `systemd/` — systemd service files
- `kiosk-browser-wrapper.sh` — auto-detects `chromium`/`chromium-browser` for the kiosk service

## Quick install

```bash
# 1. Extract the archive
tar xzf pi-kiosk-<arch>.tar.gz
cd pi-kiosk-<arch>

# 2. Run the install script (requires root)
sudo ./install.sh

# 3. Start the services
sudo systemctl start pi-kiosk-priv pi-kiosk-web

# 4. Access the dashboard
# Open a browser on the device or from another machine:
# http://<device-ip-address>:3000
```

## What these binaries are

These are Linux ELF binaries for the target architecture noted in the archive name.

Supported archive names:
- `pi-kiosk-x86_64.tar.gz` — 64-bit x86_64 (Debian 12 / Ubuntu 24.04 / KVM / QEMU / native)
- `pi-kiosk-arm64.tar.gz` — 64-bit ARM (Raspberry Pi OS 64-bit / aarch64)
- `pi-kiosk-armhf.tar.gz` — 32-bit ARM hard-float (Raspberry Pi OS 32-bit / armv7)

## System requirements

- Raspberry Pi 3B+/4B/5 (arm64/armhf) or x86_64 PC/VM
- Linux: Raspberry Pi OS, Debian 12, or Ubuntu 24.04
- Packages installed by the install script:
  - `wg-quick openvpn tor nftables openssl sqlite3 libssl3`
  - `usb-modeswitch modemmanager libqmi-utils libmbim-utils`
  - `alsa-utils libasound2 gpsd gpsd-clients`

## Hardware support (all optional — graceful degradation)

- SIM7600H-G 4G modem (cellular data, SMS, failover)
- PoE HAT (fan control via DT overlay)
- HaLo HAT (GPS NMEA + LoRa radio)
- Audio/Mic HAT (ALSA detection + monitoring)
- CSI camera (MJPEG streaming, motion/person detection)

If a HAT is not present, the system continues running normally.
