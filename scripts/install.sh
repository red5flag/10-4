#!/bin/bash
set -euo pipefail

# Pi-Kiosk install/deploy script
# Builds and installs all binaries, systemd units, and directory structure

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
SERVICE_DIR="${SERVICE_DIR:-/etc/systemd/system}"
DATA_DIR="${DATA_DIR:-/var/lib/pi-kiosk}"
CONFIG_DIR="${CONFIG_DIR:-/etc/pi-kiosk}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Pi-Kiosk Install ==="

# Create user if it doesn't exist
if ! id "pikiosk" &>/dev/null; then
    echo "Creating pikiosk user…"
    useradd --system --no-create-home --shell /usr/sbin/nologin pikiosk
fi

# Create directories
echo "Creating directories…"
mkdir -p "$DATA_DIR/clips"
mkdir -p "$DATA_DIR/backups"
mkdir -p "$CONFIG_DIR/vpn"
mkdir -p "$CONFIG_DIR/tor"
mkdir -p "$CONFIG_DIR/tls"

# Build
echo "Building binaries…"
cd "$SCRIPT_DIR/.."
cargo build --release --bin pi-kiosk-web --bin pi-kiosk-priv

# Install binaries
echo "Installing binaries…"
install -m 755 target/release/pi-kiosk-web "$INSTALL_PREFIX/bin/pi-kiosk-web"
install -m 755 target/release/pi-kiosk-priv "$INSTALL_PREFIX/bin/pi-kiosk-priv"

# Install systemd units
echo "Installing systemd units…"
install -m 644 systemd/pi-kiosk-web.service "$SERVICE_DIR/pi-kiosk-web.service"
install -m 644 systemd/pi-kiosk-priv.service "$SERVICE_DIR/pi-kiosk-priv.service"
install -m 644 systemd/kiosk-browser.service "$SERVICE_DIR/kiosk-browser.service"

# Set ownership
echo "Setting ownership…"
chown -R pikiosk:pikiosk "$DATA_DIR"
chown -R root:root "$CONFIG_DIR"

# Enable services
echo "Enabling services…"
systemctl daemon-reload
systemctl enable pi-kiosk-priv.service
systemctl enable pi-kiosk-web.service

echo ""
echo "=== Install complete ==="
echo "Start with: systemctl start pi-kiosk-priv pi-kiosk-web"
echo "Enable kiosk browser: systemctl enable kiosk-browser.service"
echo "Data directory: $DATA_DIR"
echo "Config directory: $CONFIG_DIR"
