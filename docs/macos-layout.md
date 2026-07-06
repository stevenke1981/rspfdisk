# macos-layout.md — macOS 快速分區規格

## 原則

macOS 目標碟使用 GPT / GUID Partition Map。

第一版不直接格式化 APFS；只建立正確類型與容量的目標分區，讓 macOS 安裝程式或 Disk Utility 接手格式化。

## macOS APFS 目標碟

```text
Table: GPT
Boot: UEFI

1. EFI
   Size: 512MiB
   FS: FAT32
   Type: ESP

2. macOS
   Size: fill
   FS: none
   Type: Apple APFS
   Note: 交給 macOS 格式化
```

## macOS + 共用資料

```text
1. EFI
   Size: 512MiB
   FS: FAT32

2. macOS
   Size: user input, default 300GiB
   FS: none
   Type: Apple APFS

3. Shared Data
   Size: fill
   FS: exFAT 或 no-format
   Type: Basic Data / exFAT use case
```

## macOS + Windows 預留

```text
1. EFI
2. macOS APFS
3. Windows Reserved Space
```

此模板只預留空間，不直接建立完整 Boot Camp 結構。

## 注意事項

- 不宣稱支援 Hackintosh。
- 不處理 OpenCore 安裝。
- 不處理 APFS volume group。
- 不處理 FileVault。
- 不處理 macOS Recovery 建立。
