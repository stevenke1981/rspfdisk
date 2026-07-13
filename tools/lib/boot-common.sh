#!/usr/bin/env bash
# Shared helpers for boot media scripts.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
BOOT_DIR="${ROOT_DIR}/boot"
INITRAMFS_ROOT="${BOOT_DIR}/initramfs"
GRUB_CFG="${BOOT_DIR}/grub/grub.cfg"
MEDIA_MARKER="${BOOT_DIR}/rspfdisk-media"
TEMPLATES_DIR="${ROOT_DIR}/templates"
BUILD_DIR="${ROOT_DIR}/target/boot-staging"
ISO_OUTPUT="${DIST_DIR}/rspfdisk-boot.iso"
USB_OUTPUT="${DIST_DIR}/rspfdisk-boot-usb.img"

log() { printf '[boot] %s\n' "$*" >&2; }
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
    [ -x "${bin_dir}/rspfdisk-tui" ] || die "rspfdisk-tui binary not found after build"
    echo "${bin_dir}"
}

resolve_busybox() {
    local candidate="${BUSYBOX:-}"
    if [[ -n "${candidate}" ]]; then
        [[ -x "${candidate}" ]] || die "BusyBox is not executable: ${candidate}"
        echo "${candidate}"
        return
    fi

    candidate="$(command -v busybox || true)"
    [[ -n "${candidate}" ]] || die "missing command: busybox (install busybox-static or set BUSYBOX=/path/to/busybox)"
    [[ -x "${candidate}" ]] || die "BusyBox is not executable: ${candidate}"
    echo "${candidate}"
}

busybox_has_applet() {
    local busybox="$1"
    local applet="$2"
    local applets
    local listed

    if ! applets="$("${busybox}" --list 2>&1)"; then
        die "cannot enumerate BusyBox applets: ${busybox}"
    fi
    while IFS= read -r listed; do
        [[ "${listed}" == "${applet}" ]] && return 0
    done <<<"${applets}"
    return 1
}

install_busybox() {
    local busybox="$1"
    local stage="$2"
    local applet
    local -a required_applets=(sh mount basename)

    for applet in "${required_applets[@]}"; do
        busybox_has_applet "${busybox}" "${applet}" || \
            die "BusyBox at ${busybox} lacks required applet: ${applet}"
    done

    mkdir -p "${stage}/bin"
    cp -L "${busybox}" "${stage}/bin/busybox"
    for applet in "${required_applets[@]}"; do
        ln -s busybox "${stage}/bin/${applet}"
    done
}

copy_runtime_file() {
    local stage="$1"
    local source="$2"
    local destination

    [[ "${source}" == /* ]] || die "runtime dependency is not an absolute path: ${source}"
    [[ -f "${source}" ]] || die "runtime dependency not found: ${source}"
    destination="${stage}${source}"
    mkdir -p "$(dirname "${destination}")"
    cp -L "${source}" "${destination}"
}

copy_runtime_dependencies() {
    local stage="$1"
    shift
    local -a pending=("$@")
    local -A visited=()
    local binary output line dependency root

    require_cmd ldd
    for root in "${pending[@]}"; do
        [[ -x "${root}" ]] || die "runtime binary is not executable: ${root}"
    done

    while [[ "${#pending[@]}" -gt 0 ]]; do
        binary="${pending[0]}"
        pending=("${pending[@]:1}")
        [[ -n "${visited[${binary}]+seen}" ]] && continue
        visited["${binary}"]=1

        [[ -f "${binary}" ]] || die "runtime dependency is not a file: ${binary}"
        if ! output="$(ldd "${binary}" 2>&1)"; then
            case "${output}" in
                *"not a dynamic executable"* | *"statically linked"*)
                    continue
                    ;;
                *)
                    die "cannot inspect runtime dependencies for ${binary}: ${output}"
                    ;;
            esac
        fi
        [[ "${output}" != *"not found"* ]] || \
            die "unresolved runtime dependency for ${binary}: ${output}"

        while IFS= read -r line; do
            if [[ "${line}" == *" => "* ]]; then
                dependency="${line#* => }"
                dependency="${dependency%% *}"
            elif [[ "${line}" == /* ]]; then
                dependency="${line%% *}"
            else
                continue
            fi

            [[ "${dependency}" == /* ]] || continue
            copy_runtime_file "${stage}" "${dependency}"
            [[ -n "${visited[${dependency}]+seen}" ]] || pending+=("${dependency}")
        done <<<"${output}"
    done
}

assemble_initramfs_tree() {
    local bin_dir="$1"
    local stage="${BUILD_DIR}/initramfs-root"
    local busybox

    busybox="$(resolve_busybox)"
    rm -rf "${stage}"
    mkdir -p \
        "${stage}/usr/bin" \
        "${stage}/templates" \
        "${stage}/docs" \
        "${stage}/dev" \
        "${stage}/proc" \
        "${stage}/sys" \
        "${stage}/run"

    cp "${INITRAMFS_ROOT}/init" "${stage}/init"
    chmod +x "${stage}/init"
    cp "${bin_dir}/rspfdisk" "${stage}/usr/bin/"
    cp "${bin_dir}/rspfdisk-tui" "${stage}/usr/bin/"
    install_busybox "${busybox}" "${stage}"
    copy_runtime_dependencies \
        "${stage}" \
        "${bin_dir}/rspfdisk" \
        "${bin_dir}/rspfdisk-tui" \
        "${busybox}"
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
