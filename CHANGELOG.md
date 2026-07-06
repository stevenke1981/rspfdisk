# Changelog

All notable changes to rust-spfdisk are documented in this file.

## [0.1.0] - 2026-07-06

### Added

- Rust workspace with 10 crates: core, disk, mbr, gpt, layouts, backup, safety, cli, tui, uefi
- `rspfdisk` CLI: list, inspect, backup, restore (dry-run), layout, tui
- MBR/GPT read parsers with CRC validation and 1 MiB alignment checks
- Quick layout templates for Windows, macOS, and Linux (TOML-based)
- Safe image-file GPT writer with WriteToken confirmation flow
- `.rspbak` backup format with manifest and SHA256 checksum
- Chinese TUI smoke interface (`rspfdisk-tui`)
- Linux `/dev` block device read support with danger assessment (Phase 8)
- Boot ISO/USB build scripts and QEMU smoke test tooling (Phase 7)
- UEFI `BOOTX64.EFI` read-only GPT viewer PoC (Phase 9)

### Safety

- Default read-only; writes require explicit flags
- Image writes require `--yes-i-know-this-is-an-image`
- Real disk writes require `--confirm <disk-id>`, backup, and root
- System disk writes require `--accept-system-disk-risk`
- Partition device nodes (e.g. `/dev/sda1`) are write-blocked

### Verification

- 45+ workspace unit/integration tests
- Image integration: layout dry-run + write/inspect
- Boot media bundle validation
- UEFI target build: `x86_64-unknown-uefi`

[0.1.0]: https://example.invalid/rust-spfdisk/releases/tag/v0.1.0