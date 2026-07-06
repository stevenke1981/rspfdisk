#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="x86_64-unknown-uefi"
OUT="${ROOT}/dist/EFI/BOOT"

echo "[uefi] adding target ${TARGET}"
rustup target add "${TARGET}"

echo "[uefi] building BOOTX64.EFI"
(
    cd "${ROOT}"
    cargo build -p rspfdisk-uefi --release --target "${TARGET}"
)

mkdir -p "${OUT}"
cp "${ROOT}/target/${TARGET}/release/BOOTX64.EFI" "${OUT}/BOOTX64.EFI"
echo "[uefi] output: ${OUT}/BOOTX64.EFI"
ls -lh "${OUT}/BOOTX64.EFI"