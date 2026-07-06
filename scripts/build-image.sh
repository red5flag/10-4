#!/bin/bash
set -euo pipefail

# Pi-Kiosk SD image build script
# Creates a custom Raspberry Pi OS image with pi-kiosk pre-installed
#
# Usage:
#   sudo bash scripts/build-image.sh              # full chroot build
#   sudo bash scripts/build-image.sh --stage-only  # just stage files, no chroot
#
# Requirements (full build on x86_64 host):
#   sudo apt install qemu-user-static binfmt-support parted util-linux
#
# You also need a base RaspiOS image. Download from:
#   https://www.raspberrypi.com/software/operating-systems/
#   Place it at build/raspios-bookworm-arm64-lite.img (or set BASE_IMAGE env)

# Configuration
IMAGE_NAME="${IMAGE_NAME:-pi-kiosk}"
BASE_IMAGE="${BASE_IMAGE:-raspios-bookworm-arm64-lite}"
WORK_DIR="${WORK_DIR:-$(pwd)/build}"
OUTPUT_DIR="${OUTPUT_DIR:-$(pwd)/images}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
STAGE_ONLY=false

if [ "${1:-}" = "--stage-only" ]; then
    STAGE_ONLY=true
fi

echo "=== Pi-Kiosk Image Build ==="
echo "Image: $IMAGE_NAME"
echo "Base:  $BASE_IMAGE"
echo "Mode:  $([ "$STAGE_ONLY" = true ] && echo 'stage-only' || echo 'full chroot')"
echo ""

mkdir -p "$WORK_DIR" "$OUTPUT_DIR"

# Check architecture
HOST_ARCH=$(uname -m)
NEED_QEMU=false
if [ "$HOST_ARCH" != "aarch64" ] && [ "$HOST_ARCH" != "arm64" ]; then
    NEED_QEMU=true
fi

# Check for required tools
MISSING=()
for cmd in chroot parted losetup; do
    if ! command -v $cmd &>/dev/null; then
        MISSING+=($cmd)
    fi
done

if [ "$STAGE_ONLY" = false ] && [ "$NEED_QEMU" = true ]; then
    if ! command -v qemu-aarch64-static &>/dev/null; then
        MISSING+=("qemu-aarch64-static")
    fi
fi

if [ ${#MISSING[@]} -gt 0 ]; then
    echo "ERROR: Missing required tools: ${MISSING[*]}"
    echo ""
    echo "Install with:"
    echo "  sudo apt install qemu-user-static binfmt-support parted util-linux"
    echo ""
    echo "Or run in stage-only mode (stages files without chroot):"
    echo "  sudo bash scripts/build-image.sh --stage-only"
    exit 1
fi

# Check for release binaries
if [ ! -f "$PROJECT_DIR/target/release/pi-kiosk-web" ] || [ ! -f "$PROJECT_DIR/target/release/pi-kiosk-priv" ]; then
    echo "ERROR: Release binaries not found. Build first:"
    echo "  cargo build --release"
    exit 1
fi

# Check for base image
BASE_IMG="$WORK_DIR/${BASE_IMAGE}.img"
if [ ! -f "$BASE_IMG" ]; then
    echo "ERROR: Base image not found at $BASE_IMG"
    echo ""
    echo "Download RaspiOS Bookworm ARM64 Lite from:"
    echo "  https://www.raspberrypi.com/software/operating-systems/"
    echo ""
    echo "Extract the .img file and place it at:"
    echo "  $BASE_IMG"
    echo ""
    echo "Or set BASE_IMAGE env var to match your filename."
    exit 1
fi

# Mount the image
echo "Mounting base image…"
LOOP_DEV=$(losetup -fP --show "$BASE_IMG")
BOOT_DEV="${LOOP_DEV}p1"
ROOT_DEV="${LOOP_DEV}p2"

if [ ! -b "$ROOT_DEV" ]; then
    echo "ERROR: root partition $ROOT_DEV not found. Image may be corrupted."
    losetup -d "$LOOP_DEV"
    exit 1
fi

mkdir -p "$WORK_DIR/root"
mount "$ROOT_DEV" "$WORK_DIR/root"
mount "$BOOT_DEV" "$WORK_DIR/root/boot"

# Copy binaries and systemd units into the image
echo "Copying binaries…"
cp "$PROJECT_DIR/target/release/pi-kiosk-web" "$WORK_DIR/root/usr/local/bin/"
cp "$PROJECT_DIR/target/release/pi-kiosk-priv" "$WORK_DIR/root/usr/local/bin/"
chmod 755 "$WORK_DIR/root/usr/local/bin/pi-kiosk-web" "$WORK_DIR/root/usr/local/bin/pi-kiosk-priv"

echo "Copying systemd units…"
cp "$PROJECT_DIR/systemd/pi-kiosk-priv.service" "$WORK_DIR/root/etc/systemd/system/"
cp "$PROJECT_DIR/systemd/pi-kiosk-web.service" "$WORK_DIR/root/etc/systemd/system/"
cp "$PROJECT_DIR/systemd/kiosk-browser.service" "$WORK_DIR/root/etc/systemd/system/"

echo "Copying database migration…"
mkdir -p "$WORK_DIR/root/opt/pi-kiosk"
cp "$PROJECT_DIR/migrations/001_init.sql" "$WORK_DIR/root/opt/pi-kiosk/"

if [ "$STAGE_ONLY" = true ]; then
    echo ""
    echo "=== Stage-only complete ==="
    echo "Files staged in image at $WORK_DIR/root"
    echo ""
    echo "To finish manually on the Pi (or via chroot):"
    echo "  1. apt install chromium-browser wg-quick openvpn tor nftables openssl sqlite3 libssl3"
    echo "  2. useradd --system --no-create-home --shell /usr/sbin/nologin pikiosk"
    echo "  3. mkdir -p /var/lib/pi-kiosk/{clips,backups} /etc/pi-kiosk/{vpn,tor,tls}"
    echo "  4. sqlite3 /var/lib/pi-kiosk/kiosk.db < /opt/pi-kiosk/001_init.sql"
    echo "  5. systemctl enable pi-kiosk-priv pi-kiosk-web"
    echo ""
    echo "Unmount with:"
    echo "  umount $WORK_DIR/root/boot $WORK_DIR/root"
    echo "  losetup -d $LOOP_DEV"
    exit 0
fi

# Copy qemu for chroot (if cross-arch)
if [ "$NEED_QEMU" = true ]; then
    echo "Setting up qemu for cross-arch chroot…"
    cp /usr/bin/qemu-aarch64-static "$WORK_DIR/root/usr/bin/"
fi

# Mount necessary filesystems for chroot
mount --bind /proc "$WORK_DIR/root/proc"
mount --bind /sys "$WORK_DIR/root/sys"
mount --bind /dev "$WORK_DIR/root/dev"
mount --bind /dev/pts "$WORK_DIR/root/dev/pts"

# Install pi-kiosk inside chroot
echo "Installing pi-kiosk inside image…"
chroot "$WORK_DIR/root" << 'CHROOT_EOF'
set -euo pipefail

# Install dependencies
apt-get update
apt-get install -y --no-install-recommends \
    chromium-browser \
    wg-quick \
    openvpn \
    tor \
    nftables \
    openssl \
    sqlite3 \
    libssl3

# Create user
useradd --system --no-create-home --shell /usr/sbin/nologin pikiosk 2>/dev/null || true

# Create directories
mkdir -p /var/lib/pi-kiosk/clips
mkdir -p /var/lib/pi-kiosk/backups
mkdir -p /etc/pi-kiosk/vpn
mkdir -p /etc/pi-kiosk/tor
mkdir -p /etc/pi-kiosk/tls
chown -R pikiosk:pikiosk /var/lib/pi-kiosk

# Apply database migration
sqlite3 /var/lib/pi-kiosk/kiosk.db < /opt/pi-kiosk/001_init.sql

# Enable services
systemctl enable pi-kiosk-priv.service
systemctl enable pi-kiosk-web.service

# Configure read-only root considerations
echo "tmpfs /var/lib/pi-kiosk tmpfs mode=0755,size=256M 0 0" >> /etc/fstab

echo "Chroot setup complete."
CHROOT_EOF

# Cleanup chroot
echo "Cleaning up…"
if [ "$NEED_QEMU" = true ]; then
    rm -f "$WORK_DIR/root/usr/bin/qemu-aarch64-static"
fi
umount "$WORK_DIR/root/dev/pts" "$WORK_DIR/root/dev" "$WORK_DIR/root/sys" "$WORK_DIR/root/proc"
umount "$WORK_DIR/root/boot" "$WORK_DIR/root"
losetup -d "$LOOP_DEV"

# Create output image
echo "Creating output image…"
cp "$BASE_IMG" "$OUTPUT_DIR/${IMAGE_NAME}.img"

echo ""
echo "=== Image build complete ==="
echo "Output: $OUTPUT_DIR/${IMAGE_NAME}.img"
echo ""
echo "Flash with:"
echo "  dd if=$OUTPUT_DIR/${IMAGE_NAME}.img of=/dev/sdX bs=4M status=progress"
echo ""
echo "Or use Raspberry Pi Imager to flash the .img file."
