#!/bin/bash
set -euo pipefail

# Build dist archives for all supported architectures.
#
# Usage:
#   ./scripts/build-dist.sh              # build for current host
#   ./scripts/build-dist.sh x86_64       # build x86_64 archive
#   ./scripts/build-dist.sh arm64        # build arm64 archive
#   ./scripts/build-dist.sh armhf        # build armhf archive
#   ./scripts/build-dist.sh all          # build all available archives

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist"

cd "$PROJECT_DIR"

ARCH="${1:-}"
if [ -z "$ARCH" ]; then
    case "$(uname -m)" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        armv7l|armhf)  ARCH="armhf" ;;
        *)
            echo "ERROR: Unknown host architecture: $(uname -m)"
            exit 1
            ;;
    esac
fi

copy_dist_files() {
    local out_dir="$1"
    mkdir -p "$out_dir"
    install -m 644 "$DIST_DIR/README-install.md" "$out_dir/README.md"
    install -m 755 "$DIST_DIR/install.sh" "$out_dir/install.sh"
    install -m 644 "$DIST_DIR/001_init.sql" "$out_dir/001_init.sql"
    install -m 644 "$DIST_DIR/99-pi-kiosk-modem.rules" "$out_dir/99-pi-kiosk-modem.rules"
    install -m 644 "$DIST_DIR/config.txt" "$out_dir/config.txt"
    install -m 755 "$DIST_DIR/kiosk-browser-wrapper.sh" "$out_dir/kiosk-browser-wrapper.sh"
    mkdir -p "$out_dir/systemd"
    install -m 644 "$PROJECT_DIR/systemd/pi-kiosk-web.service" "$out_dir/systemd/"
    install -m 644 "$PROJECT_DIR/systemd/pi-kiosk-priv.service" "$out_dir/systemd/"
    install -m 644 "$PROJECT_DIR/systemd/kiosk-browser.service" "$out_dir/systemd/"
}

build_for() {
    local target_arch="$1"
    local cargo_target=""
    local bin_dir=""

    case "$target_arch" in
        x86_64)
            cargo_target="x86_64-unknown-linux-gnu"
            ;;
        arm64)
            cargo_target="aarch64-unknown-linux-gnu"
            ;;
        armhf)
            cargo_target="armv7-unknown-linux-gnueabihf"
            ;;
        *)
            echo "ERROR: Unknown arch '$target_arch'"
            return 1
            ;;
    esac

    if [ "$target_arch" = "x86_64" ] && [ "$(uname -m)" = "x86_64" ]; then
        bin_dir="$PROJECT_DIR/target/release"
    else
        bin_dir="$PROJECT_DIR/target/$cargo_target/release"
    fi

    if [ ! -f "$bin_dir/pi-kiosk-web" ] || [ ! -f "$bin_dir/pi-kiosk-priv" ]; then
        echo "ERROR: Binaries not found for $target_arch at $bin_dir"
        echo "Run: ./scripts/cross-compile.sh $target_arch"
        return 1
    fi

    local out_dir="$PROJECT_DIR/build/pi-kiosk-$target_arch"
    rm -rf "$out_dir"
    copy_dist_files "$out_dir"
    install -m 755 "$bin_dir/pi-kiosk-web" "$out_dir/pi-kiosk-web"
    install -m 755 "$bin_dir/pi-kiosk-priv" "$out_dir/pi-kiosk-priv"

    local tar_path="$PROJECT_DIR/images/pi-kiosk-$target_arch.tar.gz"
    mkdir -p "$PROJECT_DIR/images"
    rm -f "$tar_path"
    tar czf "$tar_path" -C "$PROJECT_DIR/build" "pi-kiosk-$target_arch"
    echo "Created: $tar_path"
}

if [ "$ARCH" = "all" ]; then
    for a in x86_64 arm64 armhf; do
        build_for "$a" || true
    done
else
    build_for "$ARCH"
fi
