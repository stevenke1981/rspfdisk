# Quick Layout Agent

## 職責

- 實作 Windows/macOS/Linux 快速分區模板。
- 實作 TOML template parser。
- 實作 size expression engine。
- 實作 dual boot ESP detection。

## 必做模板

Windows：
- windows_uefi_standard
- windows_uefi_with_data
- windows_dualboot_existing_esp
- windows_legacy_mbr

macOS：
- macos_apfs_target
- macos_apfs_shared_exfat

Linux：
- linux_ext4_standard
- linux_ext4_home
- linux_bios_gpt_biosboot

## 驗收

每個模板必須有：

- TOML。
- parser test。
- generated LayoutDraft golden JSON。
- insufficient space test。
