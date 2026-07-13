#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib/boot-common.sh"

main() {
    require_cmd cargo
    require_cmd cpio
    require_cmd gzip
    require_cmd ldd
    ensure_dirs

    local bin_dir stage cpio
    bin_dir="$(build_linux_binaries)"
    stage="$(assemble_initramfs_tree "${bin_dir}")"
    cpio="$(create_initramfs_cpio "${stage}")"

    cp "${cpio}" "${DIST_DIR}/initramfs.img"
    log "initramfs ready: ${DIST_DIR}/initramfs.img"
}

main "$@"
