#!/bin/bash
set -euo pipefail

# Pi-Kiosk install/deploy script
# Builds and installs all binaries, systemd units, and directory structure
#
# Supports:
#   Raspberry Pi OS Lite / Desktop (arm64, armv7)
#   Debian 12, Ubuntu 24.04
#   KVM/QEMU x86_64 testing with Pi Desktop
#   Native x86_64 development builds
#
# Usage:
#   Build + install (run as root; build drops to invoking user):
#     sudo ./scripts/install.sh
#
#   Install pre-built binaries only (no build):
#     sudo ./scripts/install.sh --no-build
#
#   Build for a specific target triplet:
#     sudo ./scripts/install.sh --target x86_64-unknown-linux-gnu
#     sudo ./scripts/install.sh --target aarch64-unknown-linux-gnu
#     sudo ./scripts/install.sh --target armv7-unknown-linux-gnueabihf

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
SERVICE_DIR="${SERVICE_DIR:-/etc/systemd/system}"
DATA_DIR="${DATA_DIR:-/var/lib/pi-kiosk}"
CONFIG_DIR="${CONFIG_DIR:-/etc/pi-kiosk}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

BUILD=true
TARGET=""
BUILD_USER=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-build)
            BUILD=false
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --target=*)
            TARGET="${1#--target=}"
            shift
            ;;
        --build-user)
            BUILD_USER="$2"
            shift 2
            ;;
        --build-user=*)
            BUILD_USER="${1#--build-user=}"
            shift
            ;;
        --prefix)
            INSTALL_PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            INSTALL_PREFIX="${1#--prefix=}"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-build                 Install existing binaries; do not build"
            echo "  --target <triple>          Build/install for a specific Rust target"
            echo "  --build-user <user>        Run build as this user (default: invoking user)"
            echo "  --prefix <path>            Installation prefix (default: /usr/local)"
            echo "  --help, -h                 Show this help"
            exit 0
            ;;
        *)
            echo "ERROR: Unknown option: $1"
            echo "Run '$0 --help' for usage."
            exit 1
            ;;
    esac
done

# Determine effective user for non-root build
if [ -n "${SUDO_USER:-}" ]; then
    BUILD_USER="${BUILD_USER:-$SUDO_USER}"
else
    BUILD_USER="${BUILD_USER:-$(id -un)}"
fi

# Detect whether we are on real Raspberry Pi hardware.
# KVM/QEMU VMs, Debian x86, and Ubuntu hosts are treated as generic Linux:
# Pi-specific steps (boot config merge, gpio/i2c groups, udev modem rules)
# are skipped or made optional there.
IS_RASPBERRY_PI=false
if [ -f /proc/device-tree/model ] && grep -qi "raspberry pi" /proc/device-tree/model 2>/dev/null; then
    IS_RASPBERRY_PI=true
fi

# Detect host architecture if no target specified
if [ -z "$TARGET" ]; then
    HOST_ARCH=$(uname -m)
    case "$HOST_ARCH" in
        x86_64|amd64)
            TARGET="x86_64-unknown-linux-gnu"
            ;;
        aarch64|arm64)
            TARGET="aarch64-unknown-linux-gnu"
            ;;
        armv7l|armhf)
            TARGET="armv7-unknown-linux-gnueabihf"
            ;;
        *)
            echo "ERROR: Unknown host architecture: $HOST_ARCH"
            echo "Specify target with --target <rust-triplet>"
            exit 1
            ;;
    esac
fi

echo "=== Pi-Kiosk Install ==="
echo "Target:       $TARGET"
echo "Platform:     $([ "$IS_RASPBERRY_PI" = true ] && echo "Raspberry Pi" || echo "generic Linux (VM/desktop) — Pi-specific steps will be skipped")"
echo "Build:        $([ "$BUILD" = true ] && echo "yes" || echo "no")"
echo "Build user:   $BUILD_USER"
echo "Install prefix: $INSTALL_PREFIX"
echo ""

# Install phase must be root
if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: The install portion of this script must run as root."
    echo "Run with sudo, or for a local install set a writable prefix."
    exit 1
fi

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

# Build
if [ "$BUILD" = true ]; then
    echo "Building binaries as user '$BUILD_USER'…"
    cd "$PROJECT_DIR"

    if [ "$BUILD_USER" = "root" ] && [ -n "${SUDO_USER:-}" ]; then
        echo "WARNING: Attempting to build as root while sudo is available."
        echo "Cargo/rustup should not run as root. Use --build-user <name> or run without sudo."
    fi

    if [ "$TARGET" = "x86_64-unknown-linux-gnu" ] && [ "$(uname -m)" = "x86_64" ]; then
        # Native build
        if su -s /bin/bash "$BUILD_USER" -c "cd '$PROJECT_DIR' && cargo build --release --bin pi-kiosk-web --bin pi-kiosk-priv"; then
            echo "Native build succeeded."
        else
            echo "ERROR: Build failed."
            exit 1
        fi
    else
        # Cross build
        if su -s /bin/bash "$BUILD_USER" -c "cd '$PROJECT_DIR' && cargo build --release --target '$TARGET' --bin pi-kiosk-web --bin pi-kiosk-priv"; then
            echo "Cross build succeeded."
        else
            echo "ERROR: Build failed for target $TARGET."
            echo "Install the target toolchain or run with --no-build."
            exit 1
        fi
    fi
fi

# Locate binaries
BIN_DIR=""
if [ "$TARGET" = "x86_64-unknown-linux-gnu" ] && [ -f "$PROJECT_DIR/target/release/pi-kiosk-web" ] && [ -f "$PROJECT_DIR/target/release/pi-kiosk-priv" ]; then
    BIN_DIR="$PROJECT_DIR/target/release"
elif [ -f "$PROJECT_DIR/target/$TARGET/release/pi-kiosk-web" ] && [ -f "$PROJECT_DIR/target/$TARGET/release/pi-kiosk-priv" ]; then
    BIN_DIR="$PROJECT_DIR/target/$TARGET/release"
elif [ -f "$SCRIPT_DIR/pi-kiosk-web" ] && [ -f "$SCRIPT_DIR/pi-kiosk-priv" ]; then
    BIN_DIR="$SCRIPT_DIR"
elif [ -f "$PROJECT_DIR/dist/pi-kiosk-web" ] && [ -f "$PROJECT_DIR/dist/pi-kiosk-priv" ]; then
    BIN_DIR="$PROJECT_DIR/dist"
fi

if [ -z "$BIN_DIR" ]; then
    echo "ERROR: Pre-built binaries not found."
    echo "Run this script without --no-build, or place binaries next to the script."
    exit 1
fi

echo "Installing binaries from $BIN_DIR…"
install -m 755 "$BIN_DIR/pi-kiosk-web" "$INSTALL_PREFIX/bin/pi-kiosk-web"
install -m 755 "$BIN_DIR/pi-kiosk-priv" "$INSTALL_PREFIX/bin/pi-kiosk-priv"

# Install browser wrapper
echo "Installing browser wrapper…"
if [ -f "$PROJECT_DIR/scripts/kiosk-browser-wrapper.sh" ]; then
    install -m 755 "$PROJECT_DIR/scripts/kiosk-browser-wrapper.sh" "$INSTALL_PREFIX/bin/pi-kiosk-browser-wrapper"
fi

# Install systemd units
echo "Installing systemd units…"
for unit in pi-kiosk-web.service pi-kiosk-priv.service kiosk-browser.service; do
    if [ -f "$PROJECT_DIR/systemd/$unit" ]; then
        install -m 644 "$PROJECT_DIR/systemd/$unit" "$SERVICE_DIR/$unit"
    else
        echo "WARNING: $PROJECT_DIR/systemd/$unit not found, skipping"
    fi
done

# Install udev rules (optional; only present in full source checkouts)
UDEV_RULE=""
for candidate in \
    "$PROJECT_DIR/udev/99-pi-kiosk-modem.rules" \
    "$PROJECT_DIR/dist/99-pi-kiosk-modem.rules" \
    "$SCRIPT_DIR/99-pi-kiosk-modem.rules"; do
    if [ -f "$candidate" ]; then
        UDEV_RULE="$candidate"
        break
    fi
done
if [ -n "$UDEV_RULE" ]; then
    echo "Installing udev rules from $UDEV_RULE…"
    mkdir -p /etc/udev/rules.d
    install -m 644 "$UDEV_RULE" /etc/udev/rules.d/99-pi-kiosk-modem.rules
    udevadm trigger --subsystem-match=tty --subsystem-match=usb 2>/dev/null || true
    udevadm trigger --subsystem-match=sound 2>/dev/null || true
else
    echo "NOTE: udev rule 99-pi-kiosk-modem.rules not found — skipping."
    echo "      (Only needed for USB modem/GPS hotplug on real hardware.)"
fi

# Apply database migration
MIGRATION="$PROJECT_DIR/migrations/001_init.sql"
if [ -f "$MIGRATION" ]; then
    echo "Applying database migration…"
    sqlite3 "$DATA_DIR/kiosk.db" < "$MIGRATION"
fi

# Merge boot config fragments — only on real Raspberry Pi hardware
if [ "$IS_RASPBERRY_PI" = true ] && [ -f "$PROJECT_DIR/config/config.txt" ]; then
    if [ -f /boot/firmware/config.txt ]; then
        echo "Merging boot config fragments…"
        cat "$PROJECT_DIR/config/config.txt" >> /boot/firmware/config.txt
    elif [ -f /boot/config.txt ]; then
        echo "Merging boot config fragments…"
        cat "$PROJECT_DIR/config/config.txt" >> /boot/config.txt
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
echo "  http://localhost:3000          (on this machine)"
echo "  http://$(hostname -I 2>/dev/null | awk '{print $1}'):3000   (from another device)"
echo ""
echo "Enable fullscreen kiosk browser (requires chromium-browser):"
echo "  sudo apt install chromium-browser"
echo "  sudo systemctl enable kiosk-browser.service"
echo "  sudo systemctl start kiosk-browser.service"
echo ""
echo "Data directory: $DATA_DIR"
echo "Config directory: $CONFIG_DIR"
