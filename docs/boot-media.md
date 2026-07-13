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
/rspfdisk-media
/bin/busybox
/bin/sh
/usr/bin/rspfdisk
/usr/bin/rspfdisk-tui
/templates/*.toml
/docs/quick-help.txt
```

initramfs 打包會建立 `/dev`、`/proc`、`/sys`、`/run`，並複製 Rust binary 需要的 ELF 動態函式庫。缺少 BusyBox、runtime dependency、kernel、`grub-mkrescue` 或 `xorriso` 時，建置必須明確失敗；不得把只有檔案內容、沒有 bootloader 的 ISO 宣稱為可開機。

GRUB2 以 `/rspfdisk-media` 專用 marker 尋找開機媒體，不使用常見的 `/boot/vmlinuz` 路徑猜測 root，避免誤選內部磁碟。

## USB Image

```text
GPT
└─ Partition 1: EFI/FAT32 ESP
   ├─ EFI/BOOT/BOOTX64.EFI (GRUB2)
   ├─ boot/vmlinuz
   ├─ boot/initramfs.img
   └─ boot/grub/grub.cfg
```

USB builder 只建立 `dist/rspfdisk-boot-usb.img`，不接受真實 USB device 路徑。使用者要把完成的 image 寫入裝置時，必須另行使用適當工具並重新確認目標。

## QEMU 驗收

正式 smoke test 使用 `tools/qemu-test.sh` / `.ps1`，將 serial output 寫入 log，且只有看到 `RSPFDISK_TUI_READY` 才通過。QEMU 在看到 marker 後保持 TUI 執行而 timeout 可以接受；在 marker 前 timeout 必須失敗。

Linux CI 會分別驗證 BIOS ISO 與 UEFI USB image；USB 路徑使用 OVMF 和 `--uefi --usb`，因此會實際經過 GPT/FAT32 ESP 與 `EFI/BOOT/BOOTX64.EFI`。

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
