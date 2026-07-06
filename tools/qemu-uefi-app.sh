#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EFI="${ROOT}/dist/EFI/BOOT/BOOTX64.EFI"
DISK="${DISK:-${ROOT}/tests/images/qemu-test.img}"
OVMF="${OVMF_FD:-/usr/share/OVMF/OVMF_CODE.fd}"
OVMF_VARS="${OVMF_VARS:-/usr/share/OVMF/OVMF_VARS.fd}"

if [[ ! -f "${EFI}" ]]; then
    echo "[uefi] BOOTX64.EFI not found; run tools/build-uefi.sh first" >&2
    exit 2
fi

if [[ ! -f "${DISK}" ]]; then
    mkdir -p "$(dirname "${DISK}")"
    dd if=/dev/zero of="${DISK}" bs=1M count=64 status=none
fi

if [[ ! -f "${OVMF}" ]]; then
    echo "[uefi] OVMF firmware not found: ${OVMF}" >&2
    exit 2
fi

echo "[uefi] QEMU UEFI app test"
echo "  EFI: ${EFI}"
echo "  Disk: ${DISK}"

qemu-system-x86_64 \
    -m 1024 \
    -drive if=pflash,format=raw,readonly=on,file="${OVMF}" \
    -drive if=pflash,format=raw,file="${OVMF_VARS}" \
    -drive format=raw,file=fat:rw:"${ROOT}/dist/EFI",media=disk,if=virtio \
    -drive format=raw,file="${DISK}",media=disk,if=virtio \
    -serial stdio \
    -display none