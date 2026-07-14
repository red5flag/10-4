#!/bin/bash
set -euo pipefail

# Cross-compile pi-kiosk binaries for multiple targets
#
# Usage:
#   ./scripts/cross-compile.sh              # native build for current host
#   ./scripts/cross-compile.sh x86_64       # x86_64 Linux (native or KVM/QEMU testing)
#   ./scripts/cross-compile.sh arm64        # 64-bit ARM (aarch64)
#   ./scripts/cross-compile.sh armhf        # 32-bit ARM (armv7) — for Pi Desktop OS 32-bit
#
# Prerequisites:
#   For native builds: Rust toolchain
#   For cross builds: cargo install cross, plus podman or docker

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

ARCH="${1:-}"

# Auto-detect architecture if not specified
if [ -z "$ARCH" ]; then
    case "$(uname -m)" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        armv7l|armhf)  ARCH="armhf" ;;
        *)
            echo "ERROR: Unknown host architecture: $(uname -m)"
            echo "Usage: $0 [x86_64|arm64|armhf]"
            exit 1
            ;;
    esac
fi

case "$ARCH" in
    x86_64|amd64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    arm64|aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    armhf|armv7)
        TARGET="armv7-unknown-linux-gnueabihf"
        ;;
    *)
        echo "ERROR: Unknown arch '$ARCH'"
        echo "Usage: $0 [x86_64|arm64|armhf]"
        exit 1
        ;;
esac

IS_NATIVE=false
if [ "$TARGET" = "x86_64-unknown-linux-gnu" ] && [ "$(uname -m)" = "x86_64" ]; then
    IS_NATIVE=true
fi

cd "$PROJECT_DIR"

if [ "$IS_NATIVE" = true ]; then
    echo "=== Native build for $TARGET ==="
    cargo build --release --bin pi-kiosk-web --bin pi-kiosk-priv
else
    echo "=== Cross-compiling pi-kiosk for $ARCH ($TARGET) ==="

    # Detect container engine
    if command -v podman &>/dev/null; then
        export CROSS_CONTAINER_ENGINE=podman
        export CROSS_CONTAINER_OPTS="--security-opt label=disable"
    elif command -v docker &>/dev/null; then
        export CROSS_CONTAINER_ENGINE=docker
    else
        echo "ERROR: Neither podman nor docker found"
        echo "Install one of:"
        echo "  sudo apt install podman docker.io  # Debian/Ubuntu"
        echo "  sudo zypper install podman         # openSUSE"
        exit 1
    fi

    # The `cross` tool supplies its own linker/sysroot inside the container.
    # Unset any stale CARGO_TARGET_*_LINKER or CC_* env vars that may point
    # to a host wrapper (e.g., a no-longer-present zig cc script).
    unset "CARGO_TARGET_$(echo "$TARGET" | tr 'a-z' 'A-Z' | tr '-' '_')_LINKER" 2>/dev/null || true
    unset "CC_$(echo "$TARGET" | tr 'a-z' 'A-Z' | tr '-' '_')" 2>/dev/null || true
    unset "CXX_$(echo "$TARGET" | tr 'a-z' 'A-Z' | tr '-' '_')" 2>/dev/null || true

    cross build --release --target "$TARGET" --bin pi-kiosk-web --bin pi-kiosk-priv
fi

echo ""
echo "=== Build complete ==="
BIN_DIR=$( [ "$IS_NATIVE" = true ] && echo "target/release" || echo "target/$TARGET/release" )
echo "Binaries at: $BIN_DIR/"
echo "  pi-kiosk-web  ($(file -b "$BIN_DIR/pi-kiosk-web" | cut -d, -f1-2))"
echo "  pi-kiosk-priv ($(file -b "$BIN_DIR/pi-kiosk-priv" | cut -d, -f1-2))"
echo ""
if [ "$IS_NATIVE" = true ]; then
    echo "Install locally:"
    echo "  sudo ./scripts/install.sh --no-build --target x86_64-unknown-linux-gnu"
else
    echo "Package for distribution:"
    echo "  tar czf pi-kiosk-$ARCH.tar.gz -C \"$BIN_DIR\" pi-kiosk-web pi-kiosk-priv"
    echo ""
    echo "Build SD image with:"
    echo "  sudo bash scripts/build-image.sh --arch $ARCH"
fi
