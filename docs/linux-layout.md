# linux-layout.md — Linux 快速分區規格

## 原則

Linux 模板提供多種常見配置，第一版以 ext4 為主，btrfs 第二階段。

## Linux UEFI ext4 單系統

```text
Table: GPT
Boot: UEFI

1. EFI System
   Size: 512MiB
   FS: FAT32
   Mount: /boot/efi

2. Linux Root
   Size: fill-minus:swap
   FS: ext4
   Mount: /

3. Linux Swap
   Size: auto:swap
   FS: swap
```

## Linux UEFI ext4 + /home

```text
1. EFI System
   Size: 512MiB
   FS: FAT32
   Mount: /boot/efi

2. Linux Root
   Size: 80GiB default
   FS: ext4
   Mount: /

3. Linux Swap
   Size: auto:swap
   FS: swap

4. Linux Home
   Size: fill
   FS: ext4
   Mount: /home
```

## Linux swapfile 模板

```text
1. EFI System 512MiB FAT32 /boot/efi
2. Linux Root fill ext4 /
```

備註：swapfile 由 Linux 安裝後建立。

## Linux BIOS + GPT + GRUB biosboot

```text
Table: GPT
Boot: Legacy BIOS

1. BIOS Boot
   Size: 1MiB
   FS: none
   Type: biosboot

2. Linux Root
   Size: 80GiB
   FS: ext4
   Mount: /

3. Linux Swap
   Size: auto:swap
   FS: swap

4. Linux Home
   Size: fill
   FS: ext4
   Mount: /home
```

## Linux 雙系統

流程：

```text
1. 偵測既有 ESP。
2. 預設沿用 ESP。
3. 在未配置空間建立 root/home/swap。
4. 不修改既有 Windows/macOS 分區。
```

## 第二階段 Linux 進階功能

- btrfs subvolumes。
- LUKS 加密。
- LVM。
- ZFS。
- systemd-boot profiles。
