# boot-media.md — 開機碟設計

## 第一階段：Linux Boot ISO

使用 minimal Linux 環境啟動，進入 `rspfdisk-tui`。

優點：

- Rust std 可用。
- 支援 NVMe/SATA/USB。
- 可用現有 mkfs 工具。
- 易於 QEMU 測試。

## 開機流程

```text
Firmware
  ↓
Bootloader
  ↓
Linux Kernel + initramfs
  ↓
init script
  ↓
rspfdisk-tui
```

## ISO 內容

```text
/boot/vmlinuz
/boot/initramfs.img
/usr/bin/rspfdisk
/usr/bin/rspfdisk-tui
/templates/*.toml
/docs/quick-help.txt
```

## USB Image

```text
Partition 1: EFI/FAT32 boot
Partition 2: readonly rootfs 或 squashfs
Partition 3: writable backups optional
```

## QEMU 驗收

BIOS：

```bash
qemu-system-x86_64 -m 1024 -cdrom rspfdisk-boot.iso -hda test.img
```

UEFI：

```bash
qemu-system-x86_64 -m 1024 -bios OVMF.fd -cdrom rspfdisk-boot.iso -hda test.img
```

## 第二階段：UEFI 原生 app

輸出：

```text
EFI/BOOT/BOOTX64.EFI
```

限制：

- `no_std`。
- TUI 能力有限。
- 磁碟 Block IO 需另外封裝。
- 第一版只做 read-only viewer。
