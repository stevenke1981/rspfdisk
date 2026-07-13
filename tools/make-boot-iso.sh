#!/usr/bin/env bash
# Build dist/rspfdisk-boot.iso (Linux/WSL only).
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

usage() {
    cat <<'EOF'
Usage: tools/make-boot-iso.sh [--kernel PATH] [--skip-build]

Environment:
  KERNEL   Path to vmlinuz (auto-detected from /boot if unset)

Requires: cargo, cpio, gzip, ldd, busybox, grub-mkrescue, xorriso
EOF
}

main() {
    local skip_build=0
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --kernel)
                KERNEL="$2"
                shift 2
                ;;
            --skip-build)
                skip_build=1
                shift
                ;;
            -h | --help)
                usage
                exit 0
                ;;
            *)
                die "unknown argument: $1"
                ;;
        esac
    done

    require_cmd cargo
    require_cmd cpio
    require_cmd gzip
    require_cmd ldd
    require_cmd grub-mkrescue
    require_cmd xorriso
    ensure_dirs

    local bin_dir stage cpio kernel
    if [[ "${skip_build}" -eq 0 ]]; then
        bin_dir="$(build_linux_binaries)"
        stage="$(assemble_initramfs_tree "${bin_dir}")"
        cpio="$(create_initramfs_cpio "${stage}")"
    else
        cpio="${DIST_DIR}/initramfs.img"
        [[ -f "${cpio}" ]] || die "missing ${cpio}; run make-initramfs.sh first"
    fi

    kernel="$(resolve_kernel)"
    log "using kernel: ${kernel}"

    cp "${kernel}" "${BUILD_DIR}/boot/vmlinuz"
    cp "${cpio}" "${BUILD_DIR}/boot/initramfs.img"
    cp "${GRUB_CFG}" "${BUILD_DIR}/boot/grub/grub.cfg"
    cp "${MEDIA_MARKER}" "${BUILD_DIR}/rspfdisk-media"

    mkdir -p "${BUILD_DIR}/EFI/BOOT"
    cp "${GRUB_CFG}" "${BUILD_DIR}/EFI/BOOT/grub.cfg" 2>/dev/null || true

    log "building bootable ISO with grub-mkrescue"
    grub-mkrescue -o "${ISO_OUTPUT}" "${BUILD_DIR}"

    log "ISO ready: ${ISO_OUTPUT}"
    ls -lh "${ISO_OUTPUT}"
}

main "$@"
