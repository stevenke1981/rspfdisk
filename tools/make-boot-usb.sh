#!/usr/bin/env bash
# Create a raw USB image with ESP partition (Linux/WSL, root not required for image file).
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

USB_SIZE_MB="${USB_SIZE_MB:-512}"

usage() {
    cat <<EOF
Usage: tools/make-boot-usb.sh [--size-mb N]

Creates ${USB_OUTPUT} with GPT + FAT32 ESP containing boot files.
Requires: cargo, parted or sfdisk, mkfs.vfat, mcopy (mtools)
EOF
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --size-mb)
                USB_SIZE_MB="$2"
                shift 2
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
    ensure_dirs

    local bin_dir stage cpio
    bin_dir="$(build_linux_binaries)"
    stage="$(assemble_initramfs_tree "${bin_dir}")"
    cpio="$(create_initramfs_cpio "${stage}")"

    log "creating USB image (${USB_SIZE_MB} MiB)"
    rm -f "${USB_OUTPUT}"
    dd if=/dev/zero of="${USB_OUTPUT}" bs=1M count="${USB_SIZE_MB}" status=none

    if command -v parted >/dev/null 2>&1; then
        parted -s "${USB_OUTPUT}" mklabel gpt
        parted -s "${USB_OUTPUT}" mkpart ESP fat32 1MiB 100%
        parted -s "${USB_OUTPUT}" set 1 esp on
    else
        require_cmd sfdisk
        printf 'label: gpt\nsize=-, type=U\n' | sfdisk "${USB_OUTPUT}"
    fi

    local loop dev
    if command -v losetup >/dev/null 2>&1; then
        loop="$(losetup --find --show --partscan "${USB_OUTPUT}")"
        dev="${loop}p1"
        sleep 1
        mkfs.vfat -F 32 -n RSPFDISK "${dev}"
        mcopy -i "${dev}" "${cpio}" ::/initramfs.img
        mcopy -i "${dev}" "${BOOT_DIR}/quick-help.txt" ::/quick-help.txt
        losetup -d "${loop}"
    else
        log "losetup not available; wrote raw image only (${USB_OUTPUT})"
        log "mount loop device manually to copy ${cpio}"
    fi

    log "USB image ready: ${USB_OUTPUT}"
}

main "$@"