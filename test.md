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
- 自動進入 TUI 或 CLI shell。
- 可 inspect 附加硬碟。

### UEFI ISO test

```bash
qemu-system-x86_64 -m 1024 -bios OVMF.fd -cdrom dist/rspfdisk-boot.iso -hda test-empty.img
```

驗收：

- UEFI 可啟動。
- 可進入工具。
- 可辨識 GPT。

## TUI Smoke Tests

- 開啟 TUI。
- 選取 image disk。
- 進入快速分區。
- 選 Windows。
- 顯示草稿。
- 返回不寫入。
- 選 Linux。
- 顯示草稿。
- 按 Q 離開。

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
