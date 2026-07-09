#!/bin/bash
set -euo pipefail

# Cross-compile pi-kiosk binaries for Raspberry Pi
#
# Usage:
#   ./scripts/cross-compile.sh              # default: arm64
#   ./scripts/cross-compile.sh arm64        # 64-bit (aarch64)
#   ./scripts/cross-compile.sh armhf        # 32-bit (armv7) — for Pi Desktop OS 32-bit
#
# Prerequisites:
#   cargo install cross
#   podman or docker installed
#
# The `cross` tool uses container images with pre-configured cross-compilation
# environments, avoiding the need to install matching sysroots on the host.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

ARCH="${1:-arm64}"

case "$ARCH" in
    arm64|aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    armhf|armv7)
        TARGET="armv7-unknown-linux-gnueabihf"
        ;;
    *)
        echo "ERROR: Unknown arch '$ARCH'"
        echo "Usage: $0 [arm64|armhf]"
        exit 1
        ;;
esac

# Detect container engine
if command -v podman &>/dev/null; then
    export CROSS_CONTAINER_ENGINE=podman
    export CROSS_CONTAINER_OPTS="--security-opt label=disable"
elif command -v docker &>/dev/null; then
    export CROSS_CONTAINER_ENGINE=docker
else
    echo "ERROR: Neither podman nor docker found"
    echo "Install one of:"
    echo "  sudo zypper install podman    # openSUSE"
    echo "  sudo apt install podman       # Debian/Ubuntu"
    exit 1
fi

echo "=== Cross-compiling pi-kiosk for $ARCH ($TARGET) using $CROSS_CONTAINER_ENGINE ==="

cd "$PROJECT_DIR"

cross build --release --target "$TARGET" --bin pi-kiosk-web --bin pi-kiosk-priv

echo ""
echo "=== Cross-compile complete ==="
echo "Binaries at: target/$TARGET/release/"
echo "  pi-kiosk-web  ($(file -b target/$TARGET/release/pi-kiosk-web | cut -d, -f1-2))"
echo "  pi-kiosk-priv ($(file -b target/$TARGET/release/pi-kiosk-priv | cut -d, -f1-2))"
echo ""
echo "Package for distribution:"
echo "  tar czf pi-kiosk-$ARCH.tar.gz -C target/$TARGET/release pi-kiosk-web pi-kiosk-priv"
echo ""
echo "Build SD image with:"
echo "  sudo bash scripts/build-image.sh --arch $ARCH"
