# test.md — 測試與驗收計畫

## 測試分層

```text
Unit Tests
↓
Golden Fixtures
↓
Property Tests
↓
Image Integration Tests
↓
TUI Smoke Tests
↓
QEMU Boot Tests
↓
Disposable Real Disk Tests
```

## 絕對測試限制

第一階段所有寫入測試只能使用 image file。

禁止在 CI 或本機測試中直接寫入：

```text
/dev/sda
/dev/nvme0n1
\\.\PhysicalDrive0
系統碟
```

## Unit Tests

### Disk Core

- 讀取單一 sector。
- 讀取多 sector。
- 超出邊界回錯。
- sector size 512 / 4096。

### MBR

- 空 MBR。
- 有效 MBR。
- protective MBR。
- active flag。
- partition type id。
- overlapping partition detect。
- EBR chain。

### GPT

- valid GPT。
- invalid signature。
- bad header CRC。
- bad entries CRC。
- missing backup header。
- primary/backup mismatch。
- partition name UTF-16LE decoding。

### Layout Engine

- Windows standard template。
- Windows + D template。
- macOS APFS target template。
- macOS shared exFAT template。
- Linux ext4 template。
- Linux home template。
- Linux biosboot template。
- Windows + Linux fresh-disk multiboot template。
- small disk rejection。
- insufficient space rejection。
- alignment validation。

## Integration Tests

### Image 建立測試

```bash
truncate -s 8G test-empty.img
rspfdisk layout windows-standard test-empty.img --dry-run
rspfdisk layout windows-standard test-empty.img --write --yes-i-know-this-is-an-image
rspfdisk inspect test-empty.img --json
```

驗收：

- GPT header 正確。
- Partition entries 正確。
- 分區沒有重疊。
- 所有 start sector 1MiB 對齊。

### Windows 模板驗收

預期：

```text
ESP: 512MiB FAT32 type=esp
MSR: 16MiB type=msr
Windows: NTFS 或 no-format type=microsoft-basic-data
Recovery: 1GiB type=windows-recovery
```

### macOS 模板驗收

預期：

```text
EFI: 512MiB FAT32
macOS: Apple APFS type
APFS 不直接格式化
```

### Linux 模板驗收

預期：

```text
EFI: 512MiB FAT32 /boot/efi
Root: ext4 /
Swap: swap
Home: ext4 /home
```

## QEMU Boot Tests

### BIOS ISO test

```bash
qemu-system-x86_64 -m 1024 -cdrom dist/rspfdisk-boot.iso -hda test-empty.img
```

驗收：

- ISO 可啟動。
- serial log 必須出現 `RSPFDISK_TUI_READY`；只有 timeout 不算通過。
- 預設自動進入導引 TUI，recovery 選單才進 CLI shell。
- 可 inspect 附加硬碟。

### UEFI ISO test

```bash
qemu-system-x86_64 -m 1024 -bios OVMF.fd -cdrom dist/rspfdisk-boot.iso -hda test-empty.img
```

驗收：

- UEFI 可啟動。
- serial log 必須出現 `RSPFDISK_TUI_READY`。
- 可進入工具。
- 可辨識 GPT。

## TUI Smoke Tests

- 開啟 TUI。
- 首頁顯示 Windows、Linux、macOS、多重開機四種情境。
- 未選目標時，選情境必須先導向磁碟/image 選擇。
- 選取 image disk。
- 進入快速分區。
- 選 Windows。
- 顯示草稿。
- 返回不寫入。
- 選 Linux。
- 顯示草稿。
- 按 Q 離開。

## Boot Bundle Tests

- initramfs 包含 `/bin/sh`、`mount`、必要 runtime 目錄與 Rust binary 的動態依賴。
- ISO 打包缺少 `grub-mkrescue` 或 `xorriso` 時明確失敗，不產生非開機 fallback。
- USB image 只寫入 `dist/` 暫存 image，含 GPT/FAT32 ESP、`BOOTX64.EFI`、kernel、initramfs 與 GRUB 設定。
- PowerShell/WSL 包裝器必須傳回 Linux builder 的失敗，不可把空檔或缺檔宣稱成功。

### TUI 自動化狀態與 image 寫入測試

- 未建立備份時，Preview 的寫入操作必須導向 BackupConfirm。
- BackupConfirm 沒有有效備份檔時不得進入 WriteConfirm。
- WriteConfirm 必須拒絕實體磁碟或不存在的 image。
- 對臨時 image 寫入 GPT 後，必須重新解析並核對分區數量、起始 LBA 與大小。

## Safety Tests

- 沒有 `--write` 不得寫入。
- 沒有 backup 不得寫入。
- 磁碟 identity 不符不得還原。
- 使用者輸入錯確認文字不得寫入。
- 系統碟偵測到時必須加強警告。

## Release Gate

每次 release 前必須附上：

```text
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
image integration test report
qemu boot test report
manual TUI smoke test report
```
