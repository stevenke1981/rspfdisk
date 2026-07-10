# todos.md — 任務清單

## P0 必做

- [x] 建立 Rust workspace。（10 crates）
- [x] 建立 `crates/rspfdisk-core`。
- [x] 建立 `crates/rspfdisk-disk`。
- [x] 建立 `crates/rspfdisk-mbr`。
- [x] 建立 `crates/rspfdisk-gpt`。
- [x] 建立 `crates/rspfdisk-layouts`。
- [x] 建立 `crates/rspfdisk-backup`。
- [x] 建立 `crates/rspfdisk-cli`。
- [x] 建立 `crates/rspfdisk-tui`。
- [x] 建立 `crates/rspfdisk-safety`。
- [x] 建立 `tests/images`。
- [x] 建立 `lessons.md`。

## Disk Core

- [x] 設計 `BlockDevice` trait。
- [x] 實作 `FileBlockDevice`。
- [x] 實作 read sector。
- [x] 實作 read sectors range。
- [x] 實作 boundary check。
- [x] 實作 read-only wrapper。
- [x] 實作 test image helper。

## MBR

- [x] Parse MBR signature。
- [x] Parse 4 primary entries。
- [x] Detect empty MBR。
- [x] Detect protective MBR。
- [x] Parse CHS but do not trust CHS。
- [x] Parse LBA start and sectors。
- [x] Parse active flag。
- [x] Parse EBR chain。
- [x] Validate overlapping partitions。
- [x] Write MBR to image file only。

## GPT

- [x] Parse GPT header。
- [x] Validate GPT signature。
- [x] Validate header CRC。
- [x] Validate partition entries CRC。
- [x] Parse partition entry GUIDs。
- [x] Decode UTF-16LE partition names。
- [x] Detect backup GPT。
- [x] Compare primary/backup GPT。
- [x] Generate GPT from layout draft。
- [x] Write GPT to image file only。
- [x] GPT writer 嚴格使用 `LayoutDraft.start_lba`，並拒絕 non-GPT、重疊、越界或未對齊草稿。

## Backup

- [x] 定義 `.rspbak` 格式。
- [x] 加入 metadata。
- [x] 加入 SHA256。
- [x] 備份 MBR。
- [x] 備份 GPT header / entries。
- [x] 還原 dry-run。
- [x] 還原前驗證 disk identity。

## Quick Layouts

- [x] TOML template parser。
- [x] Size expression parser。
- [x] 支援 `fill`。
- [x] 支援 `fill-minus:SIZE`。
- [x] 支援 `auto:swap`。
- [x] 支援 Windows UEFI 標準模板。
- [x] 支援 Windows + D 槽模板。
- [x] 支援 Windows dual boot 沿用 ESP。
- [x] 支援 macOS APFS 目標碟模板。
- [x] 支援 macOS + exFAT 共用資料模板。
- [x] 支援 Linux ext4 單系統模板。
- [x] 支援 Linux ext4 + home 模板。
- [x] 支援 Linux BIOS + GPT biosboot 模板。
- [x] `fill` / `fill-minus` 會保留後續分割區與 1MiB 對齊 gap，避免草稿尾端超出 GPT usable range。

## TUI

- [x] 主選單（含目標磁碟顯示）。
- [x] 快速分區精靈（Screen::QuickLayout）。
- [x] Windows 模板選單。
- [x] macOS 模板選單。
- [x] Linux 模板選單。
- [x] Diff 預覽（Screen::Preview 基本版）。
- [x] 磁碟列表（整合 rspfdisk-disk list，DiskList 螢幕）。
- [x] 分割區表格（解析 MBR/GPT 後顯示，PartTable 螢幕）。
- [x] 容量編輯器（Screen::SizeEditor，互動式編輯分區大小，含即時重算）。
- [x] 備份確認（Screen::BackupConfirm，建立與確認備份後才可寫入）。
- [x] 寫入確認畫面（WriteConfirm 螢幕，含確認詞輸入驗證）。
- [x] TUI 寫入流程強制先建立備份，且僅對一般 image 檔執行 GPT 寫入與讀回驗證。

## Boot Media

- [x] 設計 Linux boot ISO。
- [x] 寫 `tools/make-boot-iso.ps1` / `.sh`。
- [x] 寫 `tools/make-boot-usb.ps1` / `.sh`。
- [x] 寫 `tools/qemu-test.ps1` / `.sh`。
- [x] QEMU BIOS 測試腳本。
- [x] QEMU UEFI/OVMF 測試腳本。

## Safety

- [x] 寫 destructive command policy。
- [x] 寫 WriteToken 設計。
- [x] 寫 confirmation phrase。
- [x] 寫 disk identity check。
- [x] 寫 backup-before-write。
- [x] 寫 read-back verification。
- [x] 寫 rollback instruction generator。

## Documentation

- [x] README 更新。
- [x] spec.md 持續更新（v0.1 完整度已達標，含 CLI/TUI/安全需求全部記載）。
- [x] quick-layouts.md 完成（含 TOML DSL、size 表達式、對齊規則、雙系統策略）。
- [x] windows-layout.md 完成（含 UEFI 標準、D 槽、雙系統、Legacy BIOS 四模板）。
- [x] macos-layout.md 完成（含 APFS 目標碟、exFAT 共用、Windows 預留）。
- [x] linux-layout.md 完成（含 ext4 單系統、+home、swapfile、BIOS boot、btrfs 第二階段規劃）。
- [x] safety.md 完成。
- [x] final.md 持續更新。
- [x] 修正 i18n doctest 範例，workspace doctest 可完整通過。
- [x] 8GiB slow image-write 測試改為 `#[ignore]`，由 release gate `--include-ignored` 執行。
