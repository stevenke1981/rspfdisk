#!/usr/bin/env bash
# Non-interactive UEFI app smoke test (CI-friendly).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EFI="${ROOT}/dist/EFI/BOOT/BOOTX64.EFI"
DISK="${DISK:-${ROOT}/tests/images/qemu-uefi-smoke.img}"
OVMF="${OVMF_FD:-/usr/share/OVMF/OVMF_CODE.fd}"
OVMF_VARS="${OVMF_VARS:-/usr/share/OVMF/OVMF_VARS.fd}"
SERIAL_LOG="${ROOT}/dist/qemu-uefi-serial.log"
TIMEOUT_SEC="${TIMEOUT_SEC:-45}"

if [[ ! -f "${EFI}" ]]; then
    echo "[smoke] building BOOTX64.EFI"
    bash "${ROOT}/tools/build-uefi.sh"
fi

if [[ ! -f "${OVMF}" ]]; then
    echo "[smoke] OVMF not found: ${OVMF}" >&2
    exit 2
fi

echo "[smoke] preparing GPT test disk"
cargo build --release -p rspfdisk-cli --quiet
RSPFDISK="${ROOT}/target/release/rspfdisk"
truncate -s 8G "${DISK}"
"${RSPFDISK}" layout windows-standard "${DISK}" \
    --write --yes-i-know-this-is-an-image >/dev/null

rm -f "${SERIAL_LOG}"
mkdir -p "${ROOT}/dist"

echo "[smoke] launching QEMU UEFI (timeout ${TIMEOUT_SEC}s)"
set +e
timeout "${TIMEOUT_SEC}" qemu-system-x86_64 \
    -m 1024 \
    -drive if=pflash,format=raw,readonly=on,file="${OVMF}" \
    -drive if=pflash,format=raw,file="${OVMF_VARS}" \
    -drive format=raw,file=fat:rw:"${ROOT}/dist/EFI",media=disk,if=virtio \
    -drive format=raw,file="${DISK}",media=disk,if=virtio \
    -serial file:"${SERIAL_LOG}" \
    -display none
qemu_rc=$?
set -e

# QEMU may exit via timeout while app waits for key — acceptable for smoke.
if [[ ! -f "${SERIAL_LOG}" ]]; then
    echo "[smoke] no serial log produced" >&2
    exit 1
fi

echo "[smoke] serial log:"
cat "${SERIAL_LOG}"

grep -q "Rust SPFDisk UEFI GPT Viewer" "${SERIAL_LOG}"
grep -q "Partitions:" "${SERIAL_LOG}" || grep -q "No GPT partitions" "${SERIAL_LOG}"

echo "[smoke] UEFI GPT viewer smoke passed (qemu rc=${qemu_rc})"