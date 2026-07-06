#!/usr/bin/env bash
# Full Linux acceptance pipeline for v0.1.0.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_ISO=0
WITH_QEMU=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --with-iso) WITH_ISO=1; shift ;;
        --with-qemu) WITH_QEMU=1; shift ;;
        -h | --help)
            echo "Usage: tools/verify-linux.sh [--with-iso] [--with-qemu]"
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

cd "${ROOT}"
chmod +x tools/*.sh tools/lib/*.sh boot/initramfs/init 2>/dev/null || true

echo "=== Linux Acceptance v0.1.0 ==="

bash tools/make-release.sh --verify-only --with-uefi

echo "=== Image integration (CLI) ==="
cargo build --release -p rspfdisk-cli --quiet
IMG="${ROOT}/tests/images/acceptance.img"
truncate -s 8G "${IMG}"
target/release/rspfdisk layout windows-standard "${IMG}" --dry-run | grep -q "EFI System"
target/release/rspfdisk layout windows-standard "${IMG}" \
    --write --yes-i-know-this-is-an-image >/dev/null
target/release/rspfdisk inspect "${IMG}" --json | grep -q '"partition_table"'

echo "=== Boot media bundle ==="
bash tools/qemu-test.sh --validate-only

if [[ "${WITH_ISO}" -eq 1 ]]; then
    echo "=== Boot ISO build ==="
    if [[ -f /boot/vmlinuz ]] || ls /boot/vmlinuz-* >/dev/null 2>&1; then
        bash tools/make-boot-iso.sh
        test -f dist/rspfdisk-boot.iso
        echo "ISO size: $(du -h dist/rspfdisk-boot.iso | cut -f1)"
    else
        echo "SKIP: no kernel in /boot (install linux-image or set KERNEL=)"
    fi
fi

if [[ "${WITH_QEMU}" -eq 1 ]]; then
    echo "=== QEMU UEFI smoke ==="
    if command -v qemu-system-x86_64 >/dev/null 2>&1; then
        bash tools/qemu-uefi-smoke.sh
    else
        echo "SKIP: qemu-system-x86_64 not installed"
    fi
fi

echo "=== Linux acceptance passed ==="