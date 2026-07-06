# windows-layout.md — Windows 快速分區規格

## 原則

Windows 現代 UEFI 安裝以 GPT 為主。預設模板使用：

```text
System / ESP
MSR
Windows
Recovery
```

## Windows UEFI 標準模板

```text
Table: GPT
Boot: UEFI

1. EFI System Partition
   Size: 512MiB
   FS: FAT32
   Type: ESP

2. Microsoft Reserved Partition
   Size: 16MiB
   FS: none
   Type: MSR

3. Windows
   Size: fill-minus:1GiB
   FS: no-format 或 NTFS
   Type: Microsoft Basic Data

4. Windows Recovery
   Size: 1GiB
   FS: no-format 或 NTFS
   Type: Windows Recovery
```

## Windows + D 槽模板

```text
1. ESP        512MiB
2. MSR        16MiB
3. Windows    user input, default 180GiB
4. Recovery   1GiB
5. Data       fill
```

## Windows 雙系統模板

流程：

```text
1. 掃描既有 ESP。
2. 若有 ESP，預設沿用。
3. 建立 Windows / MSR / Recovery。
4. 不覆寫既有 Linux/macOS 分區。
5. 只處理未配置空間。
```

## Windows Legacy BIOS / MBR 模板

```text
Table: MBR
Boot: Legacy BIOS

1. System Reserved / Boot
   Size: 500MiB
   Type: NTFS / 0x07
   Active: yes

2. Windows
   Size: fill-minus:1GiB
   Type: NTFS / 0x07

3. Recovery
   Size: 1GiB
   Type: NTFS / 0x27
```

## 注意事項

- 第一版不安裝 Windows Boot Manager。
- 第一版不修改 BCD。
- 第一版可建立 ESP 與資料分區，但實際 Windows 安裝仍交給 Windows Setup。
- 若使用者選 NTFS 格式化，需 boot image 內建安全可靠的 ntfs 工具。
