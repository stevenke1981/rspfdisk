#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

MODE="${MODE:-bios}"
BOOT_SOURCE="${BOOT_SOURCE:-iso}"
TEST_IMAGE="${TEST_IMAGE:-${ROOT_DIR}/tests/images/qemu-test.img}"
TIMEOUT_SEC="${TIMEOUT_SEC:-30}"
VALIDATE_ONLY="${VALIDATE_ONLY:-0}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/dist/qemu-test-serial.log}"
SERIAL_MARKER="RSPFDISK_TUI_READY"

usage() {
    cat <<'EOF'
Usage: tools/qemu-test.sh [--bios|--uefi] [--usb] [--validate-only] [--image PATH]

Runs QEMU smoke against dist/rspfdisk-boot.iso, or the UEFI USB image with --usb.
The smoke test succeeds only when the serial log contains
RSPFDISK_TUI_READY; a timeout before that marker is a failure.
EOF
}

assert_test_image_path() {
    case "${TEST_IMAGE}" in
        /dev/* | /sys/* | /proc/* | /run/* | *PhysicalDrive*)
            die "test image must be a regular image file, not a device: ${TEST_IMAGE}"
            ;;
    esac
    if [[ -b "${TEST_IMAGE}" ]]; then
        die "test image must be a regular image file, not a block device: ${TEST_IMAGE}"
    fi
    if [[ -e "${TEST_IMAGE}" && ! -f "${TEST_IMAGE}" ]]; then
        die "test image must be a regular file: ${TEST_IMAGE}"
    fi
}

create_test_disk() {
    assert_test_image_path
    mkdir -p "$(dirname "${TEST_IMAGE}")"
    if [[ ! -f "${TEST_IMAGE}" ]]; then
        log "creating test disk image: ${TEST_IMAGE} (64 MiB)"
        dd if=/dev/zero of="${TEST_IMAGE}" bs=1M count=64 status=none
    fi
}

validate_bundle() {
    local ok=0
    for f in \
        "${ROOT_DIR}/boot/initramfs/init" \
        "${ROOT_DIR}/boot/grub/grub.cfg" \
        "${ROOT_DIR}/tools/make-boot-iso.sh" \
        "${ROOT_DIR}/tools/make-boot-usb.sh" \
        "${ROOT_DIR}/tools/qemu-test.sh"; do
        if [[ -f "${f}" ]]; then
            log "OK ${f}"
        else
            log "MISSING ${f}"
            ok=1
        fi
    done
    return "${ok}"
}

run_qemu() {
    require_cmd qemu-system-x86_64
    require_cmd timeout
    if [[ "${BOOT_SOURCE}" == "usb" ]]; then
        [[ "${MODE}" == "uefi" ]] || die "USB smoke currently requires --uefi"
        [[ -f "${USB_OUTPUT}" ]] || die "USB image not found: ${USB_OUTPUT} (run make-boot-usb.sh)"
    else
        [[ -f "${ISO_OUTPUT}" ]] || die "ISO not found: ${ISO_OUTPUT} (run make-boot-iso.sh)"
    fi

    create_test_disk
    mkdir -p "$(dirname "${SERIAL_LOG}")"
    rm -f "${SERIAL_LOG}"
    local -a cmd=(
        qemu-system-x86_64
        -m 1024
        -hda "${TEST_IMAGE}"
        -display none
        -serial "file:${SERIAL_LOG}"
        -no-reboot
    )

    if [[ "${BOOT_SOURCE}" == "usb" ]]; then
        cmd+=(
            -drive "if=none,id=rspfdisk_usb,format=raw,readonly=on,file=${USB_OUTPUT}"
            -device qemu-xhci
            -device "usb-storage,drive=rspfdisk_usb,bootindex=1"
        )
    else
        cmd+=(-cdrom "${ISO_OUTPUT}")
    fi

    if [[ "${MODE}" == "uefi" ]]; then
        local ovmf="${OVMF_FD:-/usr/share/OVMF/OVMF_CODE.fd}"
        [[ -f "${ovmf}" ]] || die "OVMF firmware not found: ${ovmf}"
        cmd+=(-bios "${ovmf}")
    fi

    log "running QEMU (${MODE}): ${cmd[*]}"
    set +e
    timeout "${TIMEOUT_SEC}" "${cmd[@]}"
    local qemu_rc=$?
    set -e

    [[ -f "${SERIAL_LOG}" ]] || die "QEMU produced no serial log: ${SERIAL_LOG}"
    log "serial output (${SERIAL_LOG}):"
    cat "${SERIAL_LOG}"

    if ! grep -Fq -- "${SERIAL_MARKER}" "${SERIAL_LOG}"; then
        if [[ "${qemu_rc}" -eq 124 ]]; then
            die "QEMU timed out after ${TIMEOUT_SEC}s before serial marker ${SERIAL_MARKER}"
        fi
        die "serial marker ${SERIAL_MARKER} not found (QEMU exit ${qemu_rc})"
    fi

    if [[ "${qemu_rc}" -ne 0 && "${qemu_rc}" -ne 124 ]]; then
        die "QEMU exited with ${qemu_rc} after serial marker ${SERIAL_MARKER}"
    fi
    if [[ "${qemu_rc}" -eq 124 ]]; then
        log "QEMU timed out after ${TIMEOUT_SEC}s after serial marker ${SERIAL_MARKER} (accepted)"
    else
        log "serial marker ${SERIAL_MARKER} observed"
    fi
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --bios) MODE=bios; shift ;;
            --uefi) MODE=uefi; shift ;;
            --usb) BOOT_SOURCE=usb; shift ;;
            --validate-only) VALIDATE_ONLY=1; shift ;;
            --image) TEST_IMAGE="$2"; shift 2 ;;
            -h | --help) usage; exit 0 ;;
            *) die "unknown argument: $1" ;;
        esac
    done

    if [[ "${VALIDATE_ONLY}" -eq 1 ]]; then
        validate_bundle
        log "boot media bundle validation passed"
        exit 0
    fi

    validate_bundle
    run_qemu
    log "QEMU smoke test finished (${MODE})"
}

main "$@"
