#!/usr/bin/env bash
# Create a raw USB image with a real UEFI GRUB2 ESP (Linux/WSL only).
# All filesystem operations use regular image files; no root or loop device is required.
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

USB_SIZE_MB="${USB_SIZE_MB:-512}"
MKFS_VFAT=""
PARTITIONER=""
TMP_OUTPUT=""
ESP_IMAGE=""
USB_STAGING_DIR=""
ESP_OFFSET_BYTES=$((1 * 1024 * 1024))

usage() {
    cat <<EOF
Usage: tools/make-boot-usb.sh [--size-mb N]

Creates ${USB_OUTPUT} with GPT + FAT32 ESP containing a GRUB2 UEFI loader,
kernel, initramfs, and boot/grub/grub.cfg. The output is always an image file;
this script never accepts or writes a real USB device.
Requires: cargo, cpio, gzip, grub-mkstandalone, grub-file,
          parted or sfdisk, mkfs.vfat or mkfs.fat, mcopy, mmd
EOF
}

require_usb_tools() {
    require_cmd cargo
    require_cmd cpio
    require_cmd gzip
    require_cmd grub-mkstandalone
    require_cmd grub-file
    require_cmd mcopy
    require_cmd mmd

    if command -v mkfs.vfat >/dev/null 2>&1; then
        MKFS_VFAT="$(command -v mkfs.vfat)"
    elif command -v mkfs.fat >/dev/null 2>&1; then
        MKFS_VFAT="$(command -v mkfs.fat)"
    else
        die "missing command: mkfs.vfat or mkfs.fat"
    fi

    if command -v parted >/dev/null 2>&1; then
        PARTITIONER=parted
    elif command -v sfdisk >/dev/null 2>&1; then
        PARTITIONER=sfdisk
    else
        die "missing partitioning runtime: parted or sfdisk"
    fi
}

create_grub_efi() {
    local out="${USB_STAGING_DIR}/BOOTX64.EFI"
    mkdir -p "${USB_STAGING_DIR}"
    rm -f "${out}"

    log "building standalone GRUB2 UEFI loader"
    grub-mkstandalone \
        -O x86_64-efi \
        -o "${out}" \
        --modules="part_gpt fat normal linux echo test search search_fs_uuid search_fs_file configfile" \
        "boot/grub/grub.cfg=${GRUB_CFG}"
    [[ -s "${out}" ]] || die "grub-mkstandalone produced no EFI loader: ${out}"
    grub-file --is-x86_64-efi "${out}" || die "generated loader is not a valid x86_64 UEFI binary: ${out}"
    echo "${out}"
}

verify_esp_files() {
    local device="$1"
    local file dest
    for file in \
        /boot/vmlinuz \
        /boot/initramfs.img \
        /boot/grub/grub.cfg \
        /EFI/BOOT/BOOTX64.EFI \
        /rspfdisk-media \
        /quick-help.txt; do
        dest="${USB_STAGING_DIR}/verify-${file//\//_}"
        rm -f "${dest}"
        mcopy -i "${device}" "::${file}" "${dest}" >/dev/null \
            || die "ESP is missing required file: ${file}"
        [[ -f "${dest}" ]] || die "mtools did not read required file: ${file}"
        rm -f "${dest}"
    done
}

cleanup() {
    local rc=$?
    if [[ -n "${TMP_OUTPUT}" && "${rc}" -ne 0 ]]; then
        rm -f "${TMP_OUTPUT}"
    fi
    if [[ -n "${ESP_IMAGE}" ]]; then
        rm -f "${ESP_IMAGE}"
    fi
    if [[ -n "${USB_STAGING_DIR}" ]]; then
        rm -f "${USB_STAGING_DIR}/BOOTX64.EFI" "${USB_STAGING_DIR}"/verify-*
        rmdir "${USB_STAGING_DIR}" >/dev/null 2>&1 || true
    fi
    exit "${rc}"
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --size-mb)
                [[ $# -ge 2 ]] || die "--size-mb requires a value"
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

    [[ "${USB_SIZE_MB}" =~ ^[1-9][0-9]*$ ]] || die "size must be a positive integer MiB: ${USB_SIZE_MB}"
    (( USB_SIZE_MB >= 64 )) || die "size must be at least 64 MiB: ${USB_SIZE_MB}"
    # Keep one MiB unused inside the ESP partition and one MiB before the
    # image end, leaving room for the backup GPT header and alignment.
    local esp_size_mb=$((USB_SIZE_MB - 3))
    local partition_end_mb=$((USB_SIZE_MB - 1))
    local esp_sectors=$((esp_size_mb * 1024 * 1024 / 512))
    require_usb_tools
    ensure_dirs

    local bin_dir stage cpio kernel efi_binary
    bin_dir="$(build_linux_binaries | tail -n 1)"
    [[ -d "${bin_dir}" ]] || die "invalid Linux binary directory from boot-common: ${bin_dir}"
    stage="$(assemble_initramfs_tree "${bin_dir}")"
    cpio="$(create_initramfs_cpio "${stage}" | tail -n 1)"
    [[ -f "${cpio}" ]] || die "initramfs was not created: ${cpio}"
    kernel="$(resolve_kernel)"
    [[ -f "${kernel}" ]] || die "kernel was not resolved: ${kernel}"

    TMP_OUTPUT="${USB_OUTPUT}.tmp.$$"
    ESP_IMAGE="${BUILD_DIR}/usb-esp.$$"
    USB_STAGING_DIR="${BUILD_DIR}/usb-efi.$$"
    trap cleanup EXIT
    rm -f "${TMP_OUTPUT}"

    efi_binary="$(create_grub_efi | tail -n 1)"

    log "creating USB image (${USB_SIZE_MB} MiB)"
    dd if=/dev/zero of="${TMP_OUTPUT}" bs=1M count="${USB_SIZE_MB}" status=none

    if [[ "${PARTITIONER}" == parted ]]; then
        parted -s "${TMP_OUTPUT}" mklabel gpt
        parted -s "${TMP_OUTPUT}" mkpart ESP fat32 1MiB "${partition_end_mb}MiB"
        parted -s "${TMP_OUTPUT}" set 1 esp on
    else
        printf 'label: gpt\nstart=2048, size=%s, type=U\n' "${esp_sectors}" | sfdisk "${TMP_OUTPUT}"
    fi

    log "creating rootless FAT32 ESP image (${esp_size_mb} MiB)"
    dd if=/dev/zero of="${ESP_IMAGE}" bs=1M count="${esp_size_mb}" status=none
    "${MKFS_VFAT}" -F 32 -n RSPFDISK "${ESP_IMAGE}"

    mmd -i "${ESP_IMAGE}" ::/boot
    mmd -i "${ESP_IMAGE}" ::/boot/grub
    mmd -i "${ESP_IMAGE}" ::/EFI
    mmd -i "${ESP_IMAGE}" ::/EFI/BOOT
    mcopy -i "${ESP_IMAGE}" "${kernel}" ::/boot/vmlinuz
    mcopy -i "${ESP_IMAGE}" "${cpio}" ::/boot/initramfs.img
    mcopy -i "${ESP_IMAGE}" "${GRUB_CFG}" ::/boot/grub/grub.cfg
    mcopy -i "${ESP_IMAGE}" "${efi_binary}" ::/EFI/BOOT/BOOTX64.EFI
    mcopy -i "${ESP_IMAGE}" "${MEDIA_MARKER}" ::/rspfdisk-media
    mcopy -i "${ESP_IMAGE}" "${BOOT_DIR}/quick-help.txt" ::/quick-help.txt
    verify_esp_files "${ESP_IMAGE}"

    log "copying verified ESP into GPT image at 1 MiB offset"
    dd if="${ESP_IMAGE}" of="${TMP_OUTPUT}" bs=1M seek=1 conv=notrunc status=none
    verify_esp_files "${TMP_OUTPUT}@@${ESP_OFFSET_BYTES}"
    sync

    rm -f "${ESP_IMAGE}"
    ESP_IMAGE=""
    mv -f "${TMP_OUTPUT}" "${USB_OUTPUT}"
    TMP_OUTPUT=""
    log "UEFI GRUB2 USB image ready: ${USB_OUTPUT}"
    ls -lh "${USB_OUTPUT}"
}

main "$@"
