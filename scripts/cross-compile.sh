#!/bin/bash
set -euo pipefail

# Cross-compile pi-kiosk binaries for Raspberry Pi
#
# Usage:
#   ./scripts/cross-compile.sh              # default: arm64
#   ./scripts/cross-compile.sh arm64        # 64-bit (aarch64)
#   ./scripts/cross-compile.sh armhf        # 32-bit (armv7)
#
# Prerequisites:
#   rustup target add aarch64-unknown-linux-gnu
#   rustup target add armv7-unknown-linux-gnueabihf
#   sudo apt install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf

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

echo "=== Cross-compiling pi-kiosk for $ARCH ($TARGET) ==="

cd "$PROJECT_DIR"

# Check target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "ERROR: Rust target $TARGET not installed"
    echo "  Run: rustup target add $TARGET"
    exit 1
fi

# Check linker
case "$TARGET" in
    aarch64-unknown-linux-gnu)
        if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
            echo "ERROR: aarch64-linux-gnu-gcc not found"
            echo "  Run: sudo apt install gcc-aarch64-linux-gnu"
            exit 1
        fi
        ;;
    armv7-unknown-linux-gnueabihf)
        if ! command -v arm-linux-gnueabihf-gcc &>/dev/null; then
            echo "ERROR: arm-linux-gnueabihf-gcc not found"
            echo "  Run: sudo apt install gcc-arm-linux-gnueabihf"
            exit 1
        fi
        ;;
esac

cargo build --release --target "$TARGET" --bin pi-kiosk-web --bin pi-kiosk-priv

echo ""
echo "=== Cross-compile complete ==="
echo "Binaries at: target/$TARGET/release/"
echo "  pi-kiosk-web"
echo "  pi-kiosk-priv"
echo ""
echo "Build image with:"
echo "  sudo bash scripts/build-image.sh --arch $ARCH"
