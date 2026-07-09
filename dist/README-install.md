# Pi-Kiosk — 32-bit ARM (Raspberry Pi 4B + Pi Desktop OS)

## What's in this archive

- `pi-kiosk-web` — web server + dashboard (ELF 32-bit ARM hard-float)
- `pi-kiosk-priv` — privileged helper (ELF 32-bit ARM hard-float)
- `001_init.sql` — SQLite database migration
- `install.sh` — installation script
- `99-pi-kiosk-modem.rules` — udev rules for modem/GPS/audio/radio
- `config.txt` — boot config fragments (DT overlays)
- `systemd/` — systemd service files

## Quick install on Raspberry Pi

```bash
# 1. Extract the archive
tar xzf pi-kiosk-armhf-32bit.tar.gz
cd pi-kiosk-armhf-32bit

# 2. Run the install script (requires root)
sudo ./install.sh

# 3. Apply the database migration
sudo sqlite3 /var/lib/pi-kiosk/kiosk.db < 001_init.sql

# 4. Install boot config fragments
cat config.txt | sudo tee -a /boot/config.txt

# 5. Install udev rules
sudo cp 99-pi-kiosk-modem.rules /etc/udev/rules.d/
sudo udevadm trigger --subsystem-match=tty --subsystem-match=usb
sudo udevadm trigger --subsystem-match=sound

# 6. Start the services
sudo systemctl start pi-kiosk-priv pi-kiosk-web
sudo systemctl enable pi-kiosk-priv pi-kiosk-web

# 7. Access the dashboard
# Open a browser on the Pi or from another machine:
# http://<pi-ip-address>:3000
```

## What these binaries are

These are **Linux ELF binaries** for `armv7-unknown-linux-gnueabihf` (32-bit ARM hard-float).
They run on Raspberry Pi OS 32-bit (Bookworm/Bullseye) on Pi 3B+/4B.

They are **NOT** Windows .exe files. Raspberry Pi runs Linux, not Windows.

## System requirements

- Raspberry Pi 4B (also works on Pi 3B+)
- Raspberry Pi OS 32-bit (Desktop or Lite)
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
