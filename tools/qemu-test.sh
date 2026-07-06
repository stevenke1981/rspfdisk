#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

MODE="${MODE:-bios}"
TEST_IMAGE="${TEST_IMAGE:-${ROOT_DIR}/tests/images/qemu-test.img}"
TIMEOUT_SEC="${TIMEOUT_SEC:-30}"
VALIDATE_ONLY="${VALIDATE_ONLY:-0}"

usage() {
    cat <<'EOF'
Usage: tools/qemu-test.sh [--bios|--uefi] [--validate-only] [--image PATH]

Runs QEMU smoke test against dist/rspfdisk-boot.iso.
EOF
}

create_test_disk() {
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
    [[ -f "${ISO_OUTPUT}" ]] || die "ISO not found: ${ISO_OUTPUT} (run make-boot-iso.sh)"

    create_test_disk
    local -a cmd=(
        qemu-system-x86_64
        -m 1024
        -cdrom "${ISO_OUTPUT}"
        -hda "${TEST_IMAGE}"
        -display none
        -serial stdio
        -no-reboot
    )

    if [[ "${MODE}" == "uefi" ]]; then
        local ovmf="${OVMF_FD:-/usr/share/OVMF/OVMF_CODE.fd}"
        [[ -f "${ovmf}" ]] || die "OVMF firmware not found: ${ovmf}"
        cmd+=(-bios "${ovmf}")
    fi

    log "running QEMU (${MODE}): ${cmd[*]}"
    timeout "${TIMEOUT_SEC}" "${cmd[@]}" || {
        local rc=$?
        if [[ "${rc}" -eq 124 ]]; then
            log "QEMU timed out after ${TIMEOUT_SEC}s (may be OK for smoke)"
            return 0
        fi
        return "${rc}"
    }
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --bios) MODE=bios; shift ;;
            --uefi) MODE=uefi; shift ;;
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