# architecture.md — 系統架構

## 高層架構

```text
CLI / TUI
  ↓
Command Controller
  ↓
Safety Layer
  ↓
Layout Engine / Manual Operations
  ↓
Partition Model
  ↓
MBR Writer / GPT Writer
  ↓
BlockDevice Abstraction
  ↓
Image File / Real Disk / UEFI Block IO
```

## 建議 Rust workspace

```text
rust-spfdisk/
├─ crates/
│  ├─ rspfdisk-core/
│  ├─ rspfdisk-disk/
│  ├─ rspfdisk-mbr/
│  ├─ rspfdisk-gpt/
│  ├─ rspfdisk-layouts/
│  ├─ rspfdisk-fs/
│  ├─ rspfdisk-backup/
│  ├─ rspfdisk-safety/
│  ├─ rspfdisk-cli/
│  ├─ rspfdisk-tui/
│  ├─ rspfdisk-bootimg/
│  └─ rspfdisk-uefi/
├─ templates/
├─ tests/
├─ tools/
└─ docs/
```

## Crate 職責

### rspfdisk-core

通用資料結構：

- `DiskInfo`
- `SectorSize`
- `PartitionTableKind`
- `PartitionEntry`
- `PartitionType`
- `LayoutDraft`
- `ChangePlan`
- `DiffReport`

### rspfdisk-disk

磁碟抽象：

- `BlockDevice`
- `WritableBlockDevice`
- `FileBlockDevice`
- `LinuxBlockDevice`
- `ReadOnlyDevice`
- `SectorBuf`

### rspfdisk-mbr

MBR/EBR：

- parser。
- validator。
- writer。
- type id mapping。
- logical partition chain。

### rspfdisk-gpt

GPT：

- header parser。
- partition entries parser。
- CRC。
- GUID mapping。
- writer。
- backup GPT repair draft。

### rspfdisk-layouts

快速分區模板引擎：

- TOML parser。
- size expression。
- OS profiles。
- template validation。
- draft generation。

### rspfdisk-safety

安全層：

- WriteToken。
- confirmation。
- backup-before-write。
- system disk detection。
- destructive command guard。

### rspfdisk-tui

中文 TUI：

- 介面狀態機。
- 選單。
- 分割表格。
- 快速分區精靈。
- 寫入確認。

## 重要資料流

### 快速分區

```text
選擇模板
  ↓
讀磁碟現況 Snapshot
  ↓
套用容量規則
  ↓
產生 LayoutDraft
  ↓
Validate
  ↓
產生 DiffReport
  ↓
建立 BackupPlan
  ↓
取得 WriteToken
  ↓
寫入
  ↓
重新讀回驗證
```

### 備份還原

```text
Read partition table
  ↓
Serialize raw table + metadata
  ↓
SHA256 checksum
  ↓
.rspbak
```
