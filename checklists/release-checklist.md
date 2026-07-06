# Release Checklist — v0.1.0

- [x] 版本號更新（workspace 0.1.0）。
- [x] CHANGELOG 更新（`CHANGELOG.md`）。
- [x] README 更新。
- [x] `cargo fmt --check`。
- [x] `cargo clippy --workspace -- -D warnings`。
- [x] `cargo test --workspace`（49 tests，含 golden layout）。
- [x] image integration tests。
- [x] `tools/make-release.ps1` / `make-release.sh` release gate。
- [x] checksum 產生（`dist/SHA256SUMS`）。
- [x] dist 檔案確認（`dist/EFI/BOOT/BOOTX64.EFI`）。
- [x] QEMU UEFI app smoke（`tools/qemu-uefi-smoke.sh`，CI: `linux-acceptance.yml`）。
- [ ] QEMU BIOS ISO boot test（需 `make-boot-iso.sh` + kernel）。
- [ ] QEMU UEFI ISO boot test（需 boot ISO）。
- [ ] TUI manual smoke test。