# spec.md — Rust SPFDisk 需求規格

## 1. 專案名稱

暫定：`rust-spfdisk`  
CLI：`rspfdisk`  
TUI：`rspfdisk-tui`

## 2. 產品定位

Rust SPFDisk 是一套可放入開機碟的現代磁碟分割與啟動輔助工具。它保留 SPFDisk 的中文、快速、直覺、開機環境可用精神，但底層支援現代電腦需要的 GPT、UEFI、NVMe、4K 對齊、備份還原與安全交易式寫入。

## 3. 目標平台

第一階段：

- Linux x86_64 CLI/TUI。
- Linux boot ISO。
- USB image。
- QEMU 測試環境。

第二階段：

- Windows CLI/TUI 檢視與 boot image 製作器。
- UEFI 原生 `BOOTX64.EFI`。
- ARM64 UEFI 可行性研究。

## 4. 核心功能

### 4.1 磁碟掃描

- 列出磁碟。
- 顯示磁碟大小。
- 顯示 logical sector size / physical sector size。
- 顯示 removable / fixed。
- 顯示 model / serial。
- 顯示 read-only 狀態。

### 4.2 分割表讀取

- MBR。
- Protective MBR。
- GPT primary header。
- GPT backup header。
- GPT partition entries。
- Extended / logical partition chain。

### 4.3 分割表檢查

- MBR signature 檢查。
- GPT header CRC 檢查。
- GPT partition entries CRC 檢查。
- primary / backup GPT 一致性檢查。
- 分割區重疊檢查。
- 磁區邊界檢查。
- 1MiB / 4K 對齊檢查。

### 4.4 備份還原

- 備份 MBR。
- 備份 GPT header + entries。
- 備份分割區摘要 JSON。
- 匯出 `.rspbak`。
- 還原前比對磁碟容量、sector size、serial。
- 支援 dry-run 還原。

### 4.5 寫入模式

- 預設唯讀。
- `--dry-run` 預覽。
- `--write` 才允許寫入。
- TUI 中必須按 `W` 並輸入磁碟代號確認。
- 寫入前自動備份。

## 5. 快速分區精靈

新增功能：快速製作 Windows、macOS、Linux 分區。

所有模板先產生草稿，不直接寫入。

### 5.1 Windows 快速分區

支援：

- Windows UEFI 標準分區。
- Windows + D 槽。
- Windows 雙系統，沿用既有 EFI。
- Windows Legacy BIOS / MBR 舊機模板。
- Windows 安裝前預分區，只建立分區、不安裝 bootloader。

預設 GPT/UEFI 模板：

```text
1. EFI System Partition    512 MiB    FAT32
2. Microsoft Reserved      16 MiB     none
3. Windows                 fill       NTFS 或不格式化
4. Windows Recovery        1 GiB      NTFS 或不格式化
```

### 5.2 macOS 快速分區

支援：

- macOS APFS 目標碟。
- macOS + exFAT 共用資料分區。
- macOS + Windows 預留空間。
- 只建立 APFS 預留分區，不格式化。

原則：

- 使用 GPT / GUID Partition Map。
- 第一版不直接格式化 APFS。
- APFS 格式化交給 macOS 安裝程式或 Disk Utility。

預設模板：

```text
1. EFI            512 MiB    FAT32
2. macOS          fill       Apple APFS type, no format
```

### 5.3 Linux 快速分區

支援：

- Linux ext4 單系統。
- Linux ext4 + 獨立 `/home`。
- Linux + swap 分區。
- Linux + swapfile 模式。
- Linux btrfs 第二階段。
- Linux 雙系統，沿用既有 EFI。
- Linux BIOS + GPT + biosboot。

預設 UEFI 模板：

```text
1. EFI System     512 MiB    FAT32    /boot/efi
2. Linux Root     80 GiB     ext4     /
3. Linux Swap     auto       swap
4. Linux Home     fill       ext4     /home
```

## 6. TUI 介面

使用 `ratatui` + `crossterm`。

主畫面：

```text
磁碟列表 → 分割表檢視 → 操作選單 → 預覽 → 寫入確認
```

快速分區入口：

```text
[F] 快速分區精靈
[1] Windows
[2] macOS
[3] Linux
[4] 自訂模板
```

## 7. CLI 介面

範例：

```bash
rspfdisk list
rspfdisk inspect /dev/sda
rspfdisk backup /dev/sda --out backup.rspbak
rspfdisk restore /dev/sda backup.rspbak --dry-run
rspfdisk layout windows-standard /dev/sda --dry-run
rspfdisk layout linux-ext4-home /dev/sda --root-size 80G --swap auto --dry-run
rspfdisk tui
```

## 8. 安全需求

- 預設唯讀。
- 所有 writer 必須需要 `WriteToken`。
- `WriteToken` 只能由 confirmation module 產生。
- 寫入前自動備份。
- 寫入後重新讀回驗證。
- 支援 rollback instruction，不保證所有情況可完全回滾。

## 9. 非目標

第一版不做：

- 完整 APFS 建立/修復。
- 完整 NTFS driver。
- 真正多重開機管理器寫入 MBR boot code。
- 無確認的一鍵格式化。
- 動態調整已存在檔案系統大小。
- 搬移分割區資料。

## 10. 成功標準

v0.1 成功標準：

- 可在 Linux 上列出磁碟與 image。
- 可讀 MBR/GPT。
- 可解析 Windows/macOS/Linux 快速模板成草稿。
- 可對 image file dry-run。
- 可對 image file 寫入 GPT 並重新讀回驗證。
- TUI 可展示分割表與快速模板預覽。
