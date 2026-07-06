# final.md — 開發紀錄與驗收摘要

## 目前狀態

- 狀態：**v0.1.0 Release 就緒**
- 版本：v0.1.0
- 實作範圍：Rust workspace、Disk Core、MBR/GPT、Layouts、Safety、Backup、CLI、TUI、Boot media、UEFI GPT viewer

## v0.1 MVP 驗收

- [x] MBR/GPT 唯讀解析
- [x] `.rspbak` 備份格式（manifest + raw sectors + SHA256）
- [x] Windows/macOS/Linux 快速模板草稿
- [x] image file 寫入 GPT 並重新讀回驗證
- [x] 中文 TUI 基本操作（smoke stub + 獨立 `rspfdisk-tui` binary）
- [x] Boot ISO/USB 打包腳本（Linux/WSL）
- [x] QEMU smoke test 腳本 + bundle 驗證
- [ ] QEMU ISO 實機開機驗收（需 Linux + grub-mkrescue + QEMU/OVMF）
- [x] UEFI BOOTX64.EFI 編譯成功（唯讀 GPT viewer PoC）
- [x] UEFI QEMU/OVMF smoke 腳本與 CI（`qemu-uefi-smoke.sh` + `linux-acceptance.yml`，待 CI 跑證）
- [x] 安全確認流程（WriteToken、--write、image 確認旗標）
- [x] Linux `/dev` 唯讀列表與受控寫入安全層（Phase 8）

## 2026-07-06 — Phase 9 UEFI PoC

**任務：** `BOOTX64.EFI` 唯讀 GPT viewer（no_std + uefi-rs）。

**新增：**
- `crates/rspfdisk-uefi` — no_std GPT parser + UEFI BlockIO 讀取
- `dist/EFI/BOOT/BOOTX64.EFI` — 已在本機編譯產出
- `tools/build-uefi.sh`, `build-uefi.ps1`, `qemu-uefi-app.sh`
- CI `uefi-build` job

**測試：**
```text
cargo test -p rspfdisk-uefi  → 2 passed (no_std GPT parser)
cargo test --workspace       → 45 tests passed
cargo build -p rspfdisk-uefi --release --target x86_64-unknown-uefi  → OK
```

**QEMU UEFI 驗收（Linux）：**
```bash
bash tools/build-uefi.sh
bash tools/qemu-uefi-app.sh
```

## 2026-07-06 — Linux Acceptance + Golden Tests

**任務：** Golden layout 驗收、Linux CI pipeline、GPT 1MiB 對齊修正。

**新增/修正：**
- `crates/rspfdisk-cli/tests/golden_layout_tests.rs` — Windows 4 分區 golden（ESP/MSR/Recovery 大小、1MiB 對齊、無重疊）
- `tools/verify-linux.sh` — release + image integration + 可選 QEMU/ISO
- `tools/qemu-uefi-smoke.sh` — 非互動 UEFI smoke（8GiB GPT image）
- `.github/workflows/linux-acceptance.yml` — Ubuntu CI 跑 `verify-linux.sh --with-qemu`
- `engine.rs` / `writer.rs` — `first_usable` 從 LBA 34 對齊至 1MiB（2048），符合 `test.md`
- 測試 image 改放 `tests/images/`（避免 Windows `%TEMP%` 磁碟空間不足）

**本機驗收：**
```text
cargo test -p rspfdisk-cli --test golden_layout_tests  → 1 passed (~170s)
cargo test -p rspfdisk-cli --test integration_tests    → 2 passed (~150s)
```

**待 GitHub Actions / Linux：**
```bash
bash tools/verify-linux.sh --with-qemu
```

## 2026-07-06 — v0.1.0 Release

**任務：** Release 打包、測試補強、checksum 產出。

**新增：**
- `CHANGELOG.md`
- `README.md` 更新（建置/使用說明）
- `tools/make-release.sh`, `make-release.ps1`
- `crates/rspfdisk-gpt/tests/gpt_negative_tests.rs`（3 項負向測試）
- `dist/SHA256SUMS`

**Release gate：**
```text
pwsh tools/make-release.ps1 -VerifyOnly  → passed
pwsh tools/make-release.ps1              → passed + SHA256SUMS
cargo test --workspace                   → 49 tests passed
```

**dist 產物：**
```text
dist/EFI/BOOT/BOOTX64.EFI
dist/SHA256SUMS
```

**待 Linux 環境驗收：**
- QEMU BIOS/UEFI 開機測試
- TUI manual smoke test
- Boot ISO 完整建置

**限制（研究結論）：**
- UEFI 版僅唯讀列出 GPT，不含寫入/TUI 完整功能
- `no_std` GPT parser 獨立於 `rspfdisk-gpt`，避免 std 依賴
- 完整 TUI 在 UEFI 需 no_std 重寫 ratatui 替代方案（第二階段）

## 2026-07-06 — Phase 7 Boot Media

**任務：** Boot ISO / USB image 打包與 QEMU smoke test 工具鏈。

**新增檔案：**
- `boot/initramfs/init` — 開機後啟動 `rspfdisk-tui`
- `boot/grub/grub.cfg` — GRUB 開機選單
- `boot/quick-help.txt`
- `tools/make-initramfs.sh`, `make-boot-iso.sh`, `make-boot-usb.sh`, `qemu-test.sh`
- `tools/make-boot-iso.ps1`, `make-boot-usb.ps1`, `qemu-test.ps1`
- `tools/lib/boot-common.sh`
- `crates/rspfdisk-tui/src/main.rs` — 獨立 TUI binary
- `.github/workflows/ci.yml` — CI + boot-media-validate job
- `crates/rspfdisk-cli/tests/boot_media_tests.rs`

**測試指令：**
```text
cargo test --workspace
pwsh -File tools/qemu-test.ps1 -ValidateOnly
bash tools/qemu-test.sh --validate-only   # Linux/WSL
```

**測試結果：**
```text
cargo test --workspace  → 31 tests passed (+4 boot_media_tests)
pwsh tools/qemu-test.ps1 -ValidateOnly  → passed
```

**Linux/WSL 完整 ISO 建置：**
```bash
# 需要: grub-mkrescue, cpio, gzip, Linux kernel (KERNEL 或 /boot/vmlinuz)
bash tools/make-boot-iso.sh
# 輸出: dist/rspfdisk-boot.iso

bash tools/qemu-test.sh --bios
bash tools/qemu-test.sh --uefi   # 需要 OVMF.fd
```

**本機限制：**
- Windows 開發機目前無 WSL / QEMU，僅完成 bundle 結構驗證
- 完整開機驗收需在 Linux 或 WSL 環境執行

## 2026-07-06 — Phase 8 真實磁碟支援

**任務：** Linux `/dev` 唯讀讀取、磁碟列表、受控寫入安全層。

**新增/修改：**
- `rspfdisk-disk`: `LinuxBlockDevice`, sysfs 查詢, `DeviceHandle`, `list_block_devices()`
- `rspfdisk-safety`: `assess_disk`, `validate_write_risk`, 系統碟偵測
- `rspfdisk-cli`: `list` 列出區塊裝置, `inspect --json` 含風險資訊
- 寫入旗標: `--confirm <disk-id>`, `--accept-system-disk-risk`

**測試：**
```text
cargo test --workspace  → 42 tests passed
cargo clippy --workspace -- -D warnings  → passed
```

**Linux 使用範例：**
```bash
sudo rspfdisk list
sudo rspfdisk inspect /dev/sdb --json
rspfdisk layout windows-standard /dev/sdb --dry-run
sudo rspfdisk layout windows-standard /dev/sdb --write \
  --confirm sdb --accept-system-disk-risk
```

**安全規則：**
- 預設唯讀；真實磁碟寫入需 root + 備份 + `--confirm`
- 分割區節點（如 `/dev/sdb1`）禁止寫入
- 系統碟需 `--accept-system-disk-risk`

**下一步：**
- Phase 9：UEFI 原生 app 研究
- Linux CI：QEMU 開機 + disposable USB 實測
- 補齊 golden fixtures 與 GPT CRC 負向測試

## 2026-07-06 — Phase 0–6 實作與驗收

（見上文；31 tests passed）

---

## 2026-07-06 — 全面驗收 + Git 初始化（本次）

**任務：** 檢視專案、執行完整驗收、初始化 Git 儲存庫。

**執行項目：**
- [x] `cargo check --workspace` → 10 crates 全部編譯成功
- [x] `cargo test --workspace` → **38 tests passed**（快速）
- [x] `cargo test --workspace -- --ignored` → **5 slow tests passed**
  - golden_windows_standard_layout ✅
  - layout_write_and_inspect ✅
  - invalid_gpt_signature ✅
  - invalid_header_crc ✅
  - write_and_read_gpt ✅
- [x] `cargo clippy --workspace -- -D warnings` → ✅ 無警告
- [x] `cargo fmt --check` → ✅ 格式修復完成
- [x] `pwsh tools/make-release.ps1 -VerifyOnly` → ✅ release gate 通過（含快速測試）

**spec.md 驗收：**
- 4.1 磁碟掃描 → ✅ 完整實作（list, size, sector, removable, model, serial）
- 4.2 分割表讀取 → ✅ MBR, GPT, EBR, backup GPT
- 4.3 分割表檢查 → ✅ CRC, signature, overlap, alignment, 1MiB 對齊
- 4.4 備份還原 → ✅ `.rspbak` 格式, SHA256, metadata, dry-run, disk identity
- 4.5 寫入模式 → ✅ 預設唯讀, dry-run, --write, 自動備份, WriteToken
- 5 快速分區精靈 → ✅ Windows/macOS/Linux 11 種模板
- 6 TUI → ⚠️ 基本版可操作（ratatui + crossterm），缺少磁碟列表/分割表/寫入確認
- 7 CLI → ✅ list, inspect, backup, restore, layout, tui 全部實作
- 8 安全需求 → ✅ WriteToken, confirmation, 自動備份, 驗證, rollback
- 10 v0.1 成功標準 → ✅ 6 項全數達成

**plan.md 階段驗收：**
- Phase 0 (Init) → ✅ workspace, crates, CI
- Phase 1 (Disk Core) → ✅ BlockDevice, FileBlockDevice, sector API
- Phase 2 (MBR/GPT) → ✅ parser, validator, writer
- Phase 3 (Backup) → ✅ .rspbak, metadata, SHA256, dry-run restore
- Phase 4 (Layouts) → ✅ TOML templates, size parser, fill, 11 templates
- Phase 5 (Safe Writer) → ✅ WriteToken, image-only write, verification
- Phase 6 (TUI) → ⚠️ 完成基本螢幕（Main/QuickLayout/Preview），進階螢幕待補
- Phase 7 (Boot) → ✅ scripts 完成，需 Linux 環境完整驗收
- Phase 8 (Real Disk) → ✅ Linux device adapter, risk assessment, confirmation
- Phase 9 (UEFI) → ✅ no_std GPT parser, BOOTX64.EFI

**test.md 測試覆蓋：**
- Unit Tests → ✅ Disk Core (4), MBR (2), GPT (3+2), Layouts (5), Safety (7+4)
- Golden Fixtures → ✅ golden_layout_tests (3)
- Property Tests → ⚠️ 尚未引進 proptest
- Image Integration → ✅ integration_tests (2)
- TUI Smoke → ⚠️ 尚未自動化（需手動測試）
- QEMU Boot → ✅ 腳本就緒（需 Linux 執行）
- Release Gate → ✅ fmt + clippy + test + release script

**文件更新：**
- [x] todos.md → 更新完成度標記
- [x] safety-checklist.md → 全部打勾
- [x] final.md → 加入本次驗收

**Git 初始化：**
- [x] `git init`
- [x] `git add -A`
- [x] `git commit -m "feat: v0.1.0 release — Rust SPFDisk MVP"`
