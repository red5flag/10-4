#!/bin/bash
set -euo pipefail

# Pi-Kiosk install script for pre-built binaries
# Use this on the target device with the dist archive (no Rust toolchain needed)
#
# Supported environments:
#   Raspberry Pi OS Lite / Desktop (arm64, armv7)
#   Debian 12
#   Ubuntu 24.04
#   KVM/QEMU x86_64 VMs (for testing)
#
# Usage:  sudo ./install.sh

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
SERVICE_DIR="${SERVICE_DIR:-/etc/systemd/system}"
DATA_DIR="${DATA_DIR:-/var/lib/pi-kiosk}"
CONFIG_DIR="${CONFIG_DIR:-/etc/pi-kiosk}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Must be root
if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: This script must be run as root (use sudo)"
    exit 1
fi

echo "=== Pi-Kiosk Install ==="

# Detect architecture
TARGET_ARCH=""
if [ -f "$SCRIPT_DIR/pi-kiosk-web" ]; then
    case "$(file -b "$SCRIPT_DIR/pi-kiosk-web" 2>/dev/null | cut -d, -f2)" in
        *x86-64*) TARGET_ARCH="x86_64" ;;
        *aarch64*) TARGET_ARCH="arm64" ;;
        *ARM*) TARGET_ARCH="armhf" ;;
    esac
fi
if [ -z "$TARGET_ARCH" ]; then
    TARGET_ARCH=$(uname -m)
fi

echo "Target architecture: $TARGET_ARCH"

# Detect whether we are on real Raspberry Pi hardware.
# In a KVM/QEMU VM or on a Debian/Ubuntu x86 machine this is false and
# Pi-specific steps (boot config merge, gpio/i2c groups) are skipped.
IS_RASPBERRY_PI=false
if [ -f /proc/device-tree/model ] && grep -qi "raspberry pi" /proc/device-tree/model 2>/dev/null; then
    IS_RASPBERRY_PI=true
fi
echo "Platform: $([ "$IS_RASPBERRY_PI" = true ] && echo "Raspberry Pi" || echo "generic Linux (VM/desktop)")"

# Create runtime user
if ! id "pikiosk" &>/dev/null; then
    echo "Creating pikiosk user…"
    useradd --system --no-create-home --shell /usr/sbin/nologin pikiosk
fi

# Add to hardware access groups (gpio/i2c exist only on Pi hardware)
usermod -aG dialout pikiosk 2>/dev/null || true
usermod -aG audio pikiosk 2>/dev/null || true
if [ "$IS_RASPBERRY_PI" = true ]; then
    usermod -aG i2c pikiosk 2>/dev/null || true
    usermod -aG gpio pikiosk 2>/dev/null || true
fi

# Create directories
echo "Creating directories…"
mkdir -p "$DATA_DIR/clips"
mkdir -p "$DATA_DIR/backups"
mkdir -p "$CONFIG_DIR/vpn"
mkdir -p "$CONFIG_DIR/tor"
mkdir -p "$CONFIG_DIR/tls"

# Install pre-built binaries (from dist archive)
echo "Installing binaries…"
if [ -f "$SCRIPT_DIR/pi-kiosk-web" ] && [ -f "$SCRIPT_DIR/pi-kiosk-priv" ]; then
    install -m 755 "$SCRIPT_DIR/pi-kiosk-web" "$INSTALL_PREFIX/bin/pi-kiosk-web"
    install -m 755 "$SCRIPT_DIR/pi-kiosk-priv" "$INSTALL_PREFIX/bin/pi-kiosk-priv"
else
    echo "ERROR: Pre-built binaries not found in $SCRIPT_DIR"
    echo "Expected: pi-kiosk-web and pi-kiosk-priv next to this script"
    exit 1
fi

# Install browser wrapper (if bundled)
if [ -f "$SCRIPT_DIR/kiosk-browser-wrapper.sh" ]; then
    echo "Installing browser wrapper…"
    install -m 755 "$SCRIPT_DIR/kiosk-browser-wrapper.sh" "$INSTALL_PREFIX/bin/pi-kiosk-browser-wrapper"
fi

# Install systemd units
echo "Installing systemd units…"
if [ -d "$SCRIPT_DIR/systemd" ]; then
    install -m 644 "$SCRIPT_DIR/systemd/pi-kiosk-web.service" "$SERVICE_DIR/pi-kiosk-web.service"
    install -m 644 "$SCRIPT_DIR/systemd/pi-kiosk-priv.service" "$SERVICE_DIR/pi-kiosk-priv.service"
    install -m 644 "$SCRIPT_DIR/systemd/kiosk-browser.service" "$SERVICE_DIR/kiosk-browser.service" 2>/dev/null || true
else
    echo "WARNING: systemd unit files not found, skipping"
fi

# Install udev rules (optional; only needed for USB modem/GPS hotplug)
if [ -f "$SCRIPT_DIR/99-pi-kiosk-modem.rules" ]; then
    echo "Installing udev rules…"
    mkdir -p /etc/udev/rules.d
    install -m 644 "$SCRIPT_DIR/99-pi-kiosk-modem.rules" /etc/udev/rules.d/99-pi-kiosk-modem.rules
    udevadm trigger --subsystem-match=tty --subsystem-match=usb 2>/dev/null || true
    udevadm trigger --subsystem-match=sound 2>/dev/null || true
else
    echo "NOTE: udev rule 99-pi-kiosk-modem.rules not bundled — skipping."
fi

# Apply database migration
MIGRATION="$SCRIPT_DIR/001_init.sql"
if [ -f "$MIGRATION" ]; then
    echo "Applying database migration…"
    sqlite3 "$DATA_DIR/kiosk.db" < "$MIGRATION"
fi

# Merge boot config fragments — only on real Raspberry Pi hardware
if [ "$IS_RASPBERRY_PI" = true ] && [ -f "$SCRIPT_DIR/config.txt" ]; then
    if [ -f /boot/firmware/config.txt ]; then
        echo "Merging boot config fragments into /boot/firmware/config.txt…"
        cat "$SCRIPT_DIR/config.txt" >> /boot/firmware/config.txt
    elif [ -f /boot/config.txt ]; then
        echo "Merging boot config fragments into /boot/config.txt…"
        cat "$SCRIPT_DIR/config.txt" >> /boot/config.txt
    fi
elif [ "$IS_RASPBERRY_PI" = false ]; then
    echo "Skipping Raspberry Pi boot config merge (not Pi hardware)."
fi

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
echo ""
echo "Start the services:"
echo "  sudo systemctl start pi-kiosk-priv pi-kiosk-web"
echo ""
echo "View the dashboard:"
echo "  http://localhost:3000          (on the device)"
echo "  http://$(hostname -I 2>/dev/null | awk '{print $1}'):3000   (from another device)"
echo ""
echo "Enable fullscreen kiosk browser (requires a Chromium-based browser):"
echo "  sudo apt install chromium-browser  # Debian/Ubuntu/RaspiOS"
echo "  sudo systemctl enable kiosk-browser.service"
echo "  sudo systemctl start kiosk-browser.service"
echo ""
echo "Data directory: $DATA_DIR"
echo "Config directory: $CONFIG_DIR"
