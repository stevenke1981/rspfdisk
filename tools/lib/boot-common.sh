#!/usr/bin/env bash
# Shared helpers for boot media scripts.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
BOOT_DIR="${ROOT_DIR}/boot"
INITRAMFS_ROOT="${BOOT_DIR}/initramfs"
GRUB_CFG="${BOOT_DIR}/grub/grub.cfg"
TEMPLATES_DIR="${ROOT_DIR}/templates"
BUILD_DIR="${ROOT_DIR}/target/boot-staging"
ISO_OUTPUT="${DIST_DIR}/rspfdisk-boot.iso"
USB_OUTPUT="${DIST_DIR}/rspfdisk-boot-usb.img"

log() { printf '[boot] %s\n' "$*"; }
die() { printf '[boot] ERROR: %s\n' "$*" >&2; exit 1; }

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

ensure_dirs() {
    mkdir -p "${DIST_DIR}" "${BUILD_DIR}/boot" "${BUILD_DIR}/grub"
}

detect_linux_target() {
    if [[ "$(uname -s)" == "Linux" ]]; then
        echo "x86_64-unknown-linux-gnu"
    else
        echo "x86_64-unknown-linux-gnu"
    fi
}

build_linux_binaries() {
    local target
    target="$(detect_linux_target)"
    log "building release binaries for ${target}"
    (
        cd "${ROOT_DIR}"
        cargo build --release -p rspfdisk-cli -p rspfdisk-tui --target "${target}"
    )
    local bin_dir="${ROOT_DIR}/target/${target}/release"
    [ -x "${bin_dir}/rspfdisk" ] || die "rspfdisk binary not found after build"
    [ -f "${bin_dir}/rspfdisk-tui" ] || die "rspfdisk-tui binary not found after build"
    echo "${bin_dir}"
}

assemble_initramfs_tree() {
    local bin_dir="$1"
    local stage="${BUILD_DIR}/initramfs-root"
    rm -rf "${stage}"
    mkdir -p "${stage}/usr/bin" "${stage}/templates" "${stage}/docs"

    cp "${INITRAMFS_ROOT}/init" "${stage}/init"
    chmod +x "${stage}/init"
    cp "${bin_dir}/rspfdisk" "${stage}/usr/bin/"
    cp "${bin_dir}/rspfdisk-tui" "${stage}/usr/bin/"
    cp -r "${TEMPLATES_DIR}/." "${stage}/templates/"
    cp "${BOOT_DIR}/quick-help.txt" "${stage}/docs/quick-help.txt"
    echo "${stage}"
}

create_initramfs_cpio() {
    local stage="$1"
    local out="${BUILD_DIR}/initramfs.img"
    log "creating initramfs cpio → ${out}"
    (
        cd "${stage}"
        find . -print0 | cpio --null -o --format=newc | gzip -9 >"${out}"
    )
    echo "${out}"
}

resolve_kernel() {
    local kernel="${KERNEL:-}"
    if [[ -n "${kernel}" && -f "${kernel}" ]]; then
        echo "${kernel}"
        return
    fi
    if [[ -f /boot/vmlinuz ]]; then
        echo /boot/vmlinuz
        return
    fi
    local latest
    latest="$(ls -1 /boot/vmlinuz-* 2>/dev/null | sort -V | tail -n1 || true)"
    if [[ -n "${latest}" && -f "${latest}" ]]; then
        echo "${latest}"
        return
    fi
    die "kernel not found; set KERNEL=/path/to/vmlinuz"
}