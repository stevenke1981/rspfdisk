# plan.md — 開發計畫

## Phase 0 — 專案初始化

目標：建立安全的 Rust workspace 與測試基礎。

任務：

- 建立 workspace。
- 建立 crates。
- 建立 CI。
- 建立測試 image fixtures。
- 建立 destructive command policy。
- 建立 docs 與 examples。

驗收：

- `cargo check --workspace` 通過。
- `cargo test --workspace` 可執行。
- 沒有真實磁碟寫入功能。

## Phase 1 — Disk Core 唯讀核心

目標：可以安全讀取 image file 與磁碟 metadata。

任務：

- `BlockDevice` trait。
- `FileBlockDevice`。
- `ReadOnlyDevice` wrapper。
- sector read API。
- size / sector size handling。
- hex dump utility。

驗收：

- 可讀 raw disk image。
- 超出邊界會回錯。
- 所有測試只使用 image file。

## Phase 2 — MBR/GPT Parser

目標：讀懂 MBR、Protective MBR、GPT。

任務：

- MBR entry parser。
- EBR logical chain parser。
- GPT header parser。
- GPT entries parser。
- CRC 驗證。
- alignment 檢查。

驗收：

- 測試 fixtures 全部通過。
- 可輸出 JSON 分割區摘要。

## Phase 3 — Backup / Restore Draft

目標：建立 `.rspbak` 格式與 dry-run 還原。

任務：

- backup file format。
- metadata。
- checksum。
- restore validator。
- dry-run diff。

驗收：

- 備份 image 成 `.rspbak`。
- 還原 dry-run 顯示差異。

## Phase 4 — Quick Layout Wizard

目標：加入 Windows/macOS/Linux 快速分區模板。

任務：

- TOML template parser。
- layout DSL。
- size expression parser。
- fill / fill-minus / auto:swap。
- GPT type mapping。
- MBR type mapping。
- dual boot ESP detection。

驗收：

- Windows standard 模板產生正確分區草稿。
- Windows + D 槽模板產生正確草稿。
- macOS APFS 目標碟不格式化 APFS。
- Linux ext4 + home 模板產生正確草稿。

## Phase 5 — Safe Writer for Images

目標：先只對 image file 寫入。

任務：

- `WritableBlockDevice`。
- `WriteToken`。
- GPT writer。
- MBR writer。
- write verification。
- rollback instruction。

驗收：

- 對空 image 建 GPT。
- 寫入後可重新讀回。
- 破壞性測試只在 image 上跑。

## Phase 6 — TUI

目標：中文 TUI 可操作主要流程。

任務：

- disk list screen。
- partition table screen。
- quick layout wizard screen。
- diff preview screen。
- backup screen。
- write confirmation screen。

驗收：

- 可在終端中操作 image。
- 可選 Windows/macOS/Linux 模板。
- 不按確認不能寫入。

## Phase 7 — Boot ISO / USB Image

目標：可產出開機環境。

任務：

- 建立 minimal Linux initramfs 設計。
- 打包 `rspfdisk-tui`。
- QEMU boot smoke test。
- ISO build script。
- USB image build script。

驗收：

- QEMU 能啟動 ISO。
- 啟動後自動進入 TUI。
- 可讀附加測試 disk image。

## Phase 8 — 真實磁碟讀取與受控寫入

目標：Linux 真實磁碟支援，仍需強確認。

任務：

- `/dev` device adapter。
- root/admin 權限檢查。
- removable disk identification。
- dangerous disk warning。
- confirmation phrase。

驗收：

- 預設只讀。
- 寫入需要多重確認。
- 可在 disposable USB 測試。

## Phase 9 — UEFI 原生研究

目標：研究 `BOOTX64.EFI` 版本。

任務：

- uefi-rs PoC。
- keyboard / screen / block io。
- no_std feasibility。
- read-only GPT viewer。

驗收：

- UEFI app 可在 QEMU OVMF 啟動。
- 只讀列出 GPT 分區。
