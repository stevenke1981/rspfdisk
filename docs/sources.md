# sources.md — 規格參考來源

這份需求包的快速分區設計主要參考下列公開文件與常見系統安裝慣例。

## Windows

Microsoft Learn — UEFI/GPT-based hard drive partitions  
https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-uefigpt-based-hard-drive-partitions?view=windows-11

重點：UEFI/GPT 預設分區順序為 System/ESP、MSR、Windows、Recovery。

## macOS

Apple Support — Partition schemes available in Disk Utility on Mac  
https://support.apple.com/guide/disk-utility/partition-schemes-dsku1c614201/mac

重點：Intel Mac 與 Apple silicon Mac 建議使用 GUID Partition Map。

## Ubuntu / Linux

Ubuntu Community Help Wiki — UEFI  
https://help.ubuntu.com/community/UEFI

Ubuntu Discourse — EFI partition in dual-boot system  
https://discourse.ubuntu.com/t/efi-partition-in-dual-boot-system/62964

重點：Linux UEFI 安裝需要 ESP，雙系統時需謹慎處理既有 ESP。
