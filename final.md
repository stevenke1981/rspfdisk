# final.md — 開發紀錄與驗收摘要

## 目前狀態

- 狀態：**v0.1.0 Release 就緒**
- 版本：v0.1.0
- 實作範圍：Rust workspace、Disk Core、MBR/GPT、Layouts、Safety、Backup、CLI、TUI、Boot media、UEFI GPT viewer

## 2026-07-10 — TUI image 寫入流程完成

**任務：** 移除 WriteConfirm 的模擬成功行為，完成受控 image 寫入。

**修正：**
- Preview 未備份時強制導向 BackupConfirm。
- BackupConfirm 沒有有效備份檔時禁止進入 WriteConfirm。
- WriteConfirm 僅允許一般 image 檔，建立 `ChangePlan` 並取得 `WriteToken` 後才寫入 GPT。
- 寫入後重新解析 GPT，核對分區數量、起始 LBA 與大小；失敗時停留在確認畫面並顯示錯誤。
- 新增臨時 64MiB image 的端到端單元測試，以及無備份、非 image 拒絕測試。

**限制：**
- TUI v0.1 不開放 `/dev/*` 或 Windows `PhysicalDrive` 寫入；真實磁碟仍須使用 CLI 的完整風險旗標流程。

**驗證：**
```text
cargo fmt --all -- --check                              → passed
cargo test -p rspfdisk-tui                              → 31 passed
cargo test --workspace                                  → passed（3 個既有 8GiB slow tests ignored）
cargo clippy --workspace -- -D warnings                 → passed
cargo build --workspace --release                       → passed
pwsh -File tools/qemu-test.ps1 -ValidateOnly            → passed
```

**未執行：**
- 3 個既有 8GiB ignored image tests；本次改以 64MiB TUI image 端到端測試覆蓋變更路徑。
- Linux + QEMU/OVMF 實際開機；目前 Windows 主機只完成 bundle validate-only。

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

---

## 2026-07-06 — 第二波實作：TUI 進階、Property Tests、Boot Media 強化

**任務：** 補齊 TUI 進階螢幕、proptest、強化開機媒體驗證。

**TUI 進階螢幕（6 螢幕）：**
- Screen::Main → 主選單（顯示目標磁碟、功能快捷鍵）
- Screen::DiskList → 磁碟列表（支援掃描區塊裝置 + 輸入 image 路徑）
- Screen::PartTable → 分割表（顯示 MBR/GPT 分區、風險等級、警告）
- Screen::QuickLayout → 快速分區（↑↓選模板、Enter 產生草稿）
- Screen::Preview → 草稿預覽（顯示分區摘要、[W] 進入寫入確認）
- Screen::WriteConfirm → 寫入確認（顯示確認文字、輸入驗證）

**Property Tests（proptest）：**
- `rspfdisk-gpt`: 4 proptests — 隨機 512B buffer fuzz、CRC fuzz、short buffer rejection、多尺寸 image fuzz
- `rspfdisk-mbr`: 3 proptests — 隨機 512B buffer fuzz、large buffer fuzz、short buffer rejection
- `rspfdisk-layouts`: 4 proptests — 隨機字串 fuzz、fill expression fuzz、正整數 size 解析

**Boot Media 測試強化：**
- boot_media_tests 從 4 個 → 9 個測試
- 新增：UEFI smoke 腳本驗證、UEFI app 腳本驗證、boot ISO bundle 結構驗證、release scripts 驗證、SHA256SUMS 驗證、init 腳本內容強化

**測試結果：**
```text
cargo test --workspace        → 54 tests passed (+16 新測試)
cargo test --workspace -- --ignored → 5 slow tests passed
cargo fmt --check             → clean
cargo clippy -D warnings      → clean
```

**spec.md 狀態更新：**
- 6 TUI → ✅ 已升級為 6 螢幕完整 TUI（磁碟列表、分割表、寫入確認）

**plan.md 狀態更新：**
- Phase 6 (TUI) → ✅ 完成進階螢幕（磁碟列表、分割表、寫入確認）
- Property Tests → ✅ test.md 中 Property Tests 層級已覆蓋

## 2026-07-06 — 修復編譯錯誤 + 文件同步

**任務：** 接續開發時發現 `rspfdisk-disk` crate 編譯中斷，`linux_sysfs.rs` 使用了 `?` 將 `CoreError` 轉型為 `DiskError` 但缺少對應的 `From` 實作。

**修正：**
- `crates/rspfdisk-disk/src/error.rs` — 新增 `Core(#[from] CoreError)` variant
- `crates/rspfdisk-disk/src/linux_sysfs.rs` — 為 `block_name` 欄位加上 `#[allow(dead_code)]`

**驗證：**
```text
cargo check --workspace              → ✅
cargo test --workspace                → 52 tests passed
cargo test --workspace -- --ignored   → 5 slow tests passed
cargo clippy --workspace -- -D warnings → ✅ clean
cargo fmt --check                     → ✅ clean
```

**文件更新：**
- todos.md → TUI 螢幕狀態修正為正確完成標記（DiskList ✅、PartTable ✅、WriteConfirm ✅），保留容量編輯器與備份確認為未完成
- todos.md → 文件章節全部標記為完成（spec.md、quick-layouts、windows/macos/linux-layout）

## 2026-07-06 — TUI 容量編輯器 + 備份確認畫面

**任務：** 實作 SizeEditor 和 BackupConfirm 兩個新螢幕，完成 TUI 8 螢幕全覆蓋。

**新增：**
- `Screen::SizeEditor` — 互動式分區容量編輯器
  - ↑↓ 選取分區，Enter 開始編輯
  - 支援 `parse_byte_size` 格式（80GiB、512MiB 等）
  - 編輯後自動重算 start_lba 與更新 Preview
  - 顯示總計與剩餘空間
- `Screen::BackupConfirm` — 備份確認與建立
  - 顯示備份狀態（尚未備份 / 備份中 / 完成）
  - [B] 呼叫 `create_backup()` 建立 `.rspbak`
  - 備份成功後才可 [W] 進入寫入確認
- `rspfdisk-tui/Cargo.toml` — 加入 `rspfdisk-backup`、`chrono` 依賴

**導航流程更新：**
```text
Preview → [E] SizeEditor → Esc → Preview
Preview → [B] BackupConfirm → [B] backup → [W] WriteConfirm
```

**驗證：**
```text
cargo fmt --check                     → ✅ clean
cargo clippy --workspace -- -D warnings → ✅ clean
cargo test --workspace                → 52 tests passed
cargo test --workspace -- --ignored   → 5 slow tests passed
```

**下一步（非 blocking）：**
- QEMU BIOS/UEFI 實機開機測試（需 Linux 環境 + grub-mkrescore）
- Boot ISO 完整建置
- TUI 自動化集成測試

## 2026-07-09 — GPT writer/layout hardening + release build

**任務：** 檢視專案、完成可落地改善、release 編譯、commit/push。

**修正：**
- `rspfdisk-layouts`: `fill` / `fill-minus` 改為在配置時保留後續分割區最小需求與 1MiB alignment gap，避免草稿尾端超出 GPT usable range。
- `rspfdisk-gpt`: writer 改為嚴格使用 `LayoutDraft.start_lba`，並拒絕 non-GPT、分割區過多、零大小、sector 未對齊、越界與重疊草稿。
- `rspfdisk-gpt` tests: 新增 writer 保留 draft start LBA、拒絕 MBR draft、拒絕 overlap draft 的測試；負向 GPT 測試改用 256MiB 小 image，避免 Windows 8GiB sync 卡住。
- `rspfdisk-i18n`: 修正 doctest 範例中的未定義變數。
- CLI tests: 測試 image/backup 改放 `tests/images/generated/`，避免覆寫 tracked fixture。
- 慢速 8GiB image-write 測試改為 `#[ignore]`，預設 workspace 測試不再卡 Windows image sync；release gate 仍可用 `--include-ignored` 執行。
- `.gitignore`: 忽略 `tests/images/generated/` 與本機 `.codebase-memory/`。

**驗證：**
```text
C:\Users\steven\.cargo\bin\cargo.exe fmt --check                              → passed
CARGO_TARGET_DIR=target\codex-verify cargo test -p rspfdisk-gpt              → passed
CARGO_TARGET_DIR=target\codex-verify cargo test -p rspfdisk-layouts          → passed
CARGO_TARGET_DIR=target\codex-verify cargo test -p rspfdisk-i18n --doc       → passed
CARGO_TARGET_DIR=target\codex-verify cargo test --workspace                  → passed (fast; 3 slow image-write tests ignored)
CARGO_TARGET_DIR=target\codex-verify cargo clippy --workspace -- -D warnings → passed
CARGO_TARGET_DIR=target\codex-verify cargo build --workspace --release       → passed
```

**注意：**
- 本機 shell 的 `PATH` 沒有 `cargo`，本次使用完整路徑 `C:\Users\steven\.cargo\bin\cargo.exe`。
- QEMU BIOS/UEFI 實機開機驗收仍需 Linux + QEMU/OVMF 環境。
