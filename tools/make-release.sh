#!/usr/bin/env bash
# Release gate: fmt, clippy, test, checksums.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERIFY_ONLY=0
BUILD_UEFI=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --verify-only) VERIFY_ONLY=1; shift ;;
        --with-uefi) BUILD_UEFI=1; shift ;;
        -h | --help)
            echo "Usage: tools/make-release.sh [--verify-only] [--with-uefi]"
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

cd "${ROOT}"

log() { printf '[release] %s\n' "$*"; }

log "cargo fmt --check"
cargo fmt --check

log "cargo clippy --workspace -- -D warnings"
cargo clippy --workspace -- -D warnings

log "cargo test --workspace (fast)"
cargo test --workspace
log "cargo test --workspace --include-ignored (slow image writes)"
cargo test --workspace -- --include-ignored

log "cargo test -p rspfdisk-uefi"
cargo test -p rspfdisk-uefi

if [[ "${BUILD_UEFI}" -eq 1 ]]; then
    log "building UEFI app"
    bash tools/build-uefi.sh
fi

if [[ "${VERIFY_ONLY}" -eq 1 ]]; then
    log "verify-only complete"
    exit 0
fi

CHECKSUMS="${ROOT}/dist/SHA256SUMS"
mkdir -p "${ROOT}/dist"
log "writing checksums → ${CHECKSUMS}"
{
    cd "${ROOT}/dist"
    if command -v sha256sum >/dev/null 2>&1; then
        find . -type f ! -name 'SHA256SUMS' -print0 | sort -z | xargs -0 sha256sum
    else
        find . -type f ! -name 'SHA256SUMS' -print0 | sort -z | xargs -0 shasum -a 256
    fi
} >"${CHECKSUMS}"

log "release gate passed"
cat "${CHECKSUMS}"