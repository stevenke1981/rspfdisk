# Bootable Guided Install

This guide explains the newcomer workflow for using SPFDisk from GRUB2 boot media. SPFDisk prepares and verifies partition layouts; it does not install an operating system or the bootloader for that operating system.

## English

### Before You Boot

Have these items ready:

- The SPFDisk GRUB2 ISO or USB media. See [boot-media.md](boot-media.md) for the media layout and boot path.
- The official installer media for the operating system you intend to install. SPFDisk media is not a replacement for it.
- A separate backup of important data. The `.rspbak` created by SPFDisk is a partition-layout backup, not a substitute for a full file backup.
- The target disk identity: model, capacity, serial or another detail that lets you distinguish it from every other disk.

If the disk contains an existing OS or data, stop and make sure the intended operation is understood before continuing. A correct layout applied to the wrong disk still causes data loss.

### 1. Boot the GRUB2 Media

1. Insert or attach the SPFDisk ISO/USB.
2. Open the computer firmware boot menu and select the SPFDisk media. Choose the UEFI or legacy entry that matches the layout you intend to use.
3. At the GRUB2 menu, choose the Rust SPFDisk guided/TUI entry. The CLI shell or recovery entry is for troubleshooting and advanced work.
4. Wait for the Linux boot environment and the `rspfdisk-tui` interface.

The GRUB2 loader on this media only starts the SPFDisk environment. It is not installed into the target disk and cannot substitute for Windows Boot Manager, Linux GRUB/systemd-boot, or macOS boot components.

### 2. Select a Scenario

Select the target disk first, inspect its current partition table, then choose one of these user-facing scenarios:

| Scenario | Typical draft | Handoff |
|----------|---------------|---------|
| Windows | GPT/UEFI, ESP, MSR, Windows target, optional recovery or data space | Boot Windows Setup |
| Linux | ESP or BIOS boot partition, root, optional home, and swap | Boot the chosen Linux installer |
| macOS | GPT/GUID Partition Map and an Apple APFS target partition | Boot macOS Installer or use Disk Utility |
| Multiboot | Shared or selected ESP plus separate space for each OS | Install each OS separately from its official media |

Scenario names describe the intended outcome. The underlying template names such as `windows_uefi_standard` or `macos_apfs_target` are useful for technical references, but they are not a promise that SPFDisk installs the named operating system.

### 3. Preview the Partition Draft

Before any write, inspect the complete draft and its diff from the current disk:

- MBR or GPT and UEFI/BIOS mode.
- Every partition's name, type, start sector, end sector, size, alignment, and filesystem note.
- Existing partitions that will be preserved.
- New, resized, or removed partitions.
- Whether an existing EFI System Partition is being reused or a new one is being prepared.

The preview is the point to change a size, choose another scenario, or cancel. A preview must not write to the target.

### 4. Create and Verify the Backup

Start the backup step before confirmation:

1. Create the `.rspbak` layout backup.
2. Store a copy away from the target disk.
3. Check the backup metadata, disk identity, sector size, and partition summary.
4. Keep the backup with the installation notes so it can be matched to the same disk later.

The backup records partition metadata and integrity information. It does not copy personal files or operating-system data.

### 5. Dry-Run and Confirm

Use the final dry-run and confirmation screen to compare the intended scenario with the actual target. Confirm only when all of the following are true:

- The model, capacity, and identity match the disk you intend to modify.
- The partition diff is understood, including any removed or repurposed space.
- The backup exists outside the target disk.
- The selected boot mode and partition table match the operating system installer.
- You have the official OS installer ready for the next step.

The safety sequence is:

```text
Snapshot -> Draft -> Preview -> Backup -> Dry Run -> Explicit Confirmation -> Write / Rollback
```

Real-disk writes remain high risk. Development and automated tests use image files. A booted live environment is not blanket permission to write a physical disk; only a build and path that explicitly expose real-disk writing and pass its safety gates may do so. If the running TUI or image path rejects a physical disk, do not bypass that restriction.

### 6. Hand Off to the OS Installer

After a permitted write completes, reboot or power off and boot the official installer, not the SPFDisk media. Keep the SPFDisk media available for inspection or recovery, but do not expect the target disk to boot an operating system yet.

#### Windows

Boot Windows Setup and select the intended prepared Windows space. Windows Setup installs Windows, performs the filesystem work it requires, creates or updates Windows Boot Manager, and handles Windows recovery configuration. SPFDisk does none of those installation tasks.

#### Linux

Boot the distribution installer. Assign the prepared ESP, root, home, and swap areas according to the selected layout and the distribution's instructions. The Linux installer formats the selected filesystems as needed and installs/configures GRUB, systemd-boot, or the distribution's chosen bootloader.

#### macOS

Use macOS Installer or Disk Utility on supported Apple hardware. SPFDisk prepares the GPT partition type and capacity only. **SPFDisk does not format APFS.** macOS must format and create the APFS container/volumes and install its boot components. SPFDisk does not configure Hackintosh/OpenCore, FileVault, or macOS Recovery.

#### Multiboot

Install one operating system at a time. Before each install, verify the target disk and partition again, preserve the other OS areas, and follow the installer-specific bootloader guidance. A shared ESP is only prepared storage; it is not a bootloader installation.

### Stop Conditions

Cancel the workflow and investigate if:

- The target disk identity is ambiguous.
- The preview differs from the intended scenario.
- A backup cannot be created or verified away from the target.
- The installer media is missing or does not match the chosen OS.
- APFS formatting is being requested from SPFDisk.
- A tool asks you to bypass a real-disk safety gate.

For layout details, see [quick-layouts.md](quick-layouts.md), [windows-layout.md](windows-layout.md), [linux-layout.md](linux-layout.md), [macos-layout.md](macos-layout.md), and [safety.md](safety.md).

## 繁體中文

### 開機前準備

請先準備：

- SPFDisk GRUB2 ISO 或 USB 媒體。媒體配置與開機路徑請參考 [boot-media.md](boot-media.md)。
- 預計安裝 OS 的官方安裝媒體。SPFDisk 媒體不能取代它。
- 重要資料的獨立備份。SPFDisk 建立的 `.rspbak` 是分割區版面備份，不是完整檔案備份的替代品。
- 目標磁碟身份：型號、容量、序號或其他能與所有磁碟區分的資訊。

若磁碟內已有 OS 或資料，請在繼續前確認操作內容。正確的版面若套用到錯誤磁碟，仍然會造成資料遺失。

### 1. 從 GRUB2 媒體開機

1. 插入或掛載 SPFDisk ISO/USB。
2. 開啟電腦韌體的開機選單，選取 SPFDisk 媒體。依預計使用的版面選擇相符的 UEFI 或 legacy 入口。
3. 在 GRUB2 選單選擇 Rust SPFDisk 導引/TUI 入口。CLI shell 或 recovery 入口供疑難排解與進階操作使用。
4. 等待 Linux 開機環境與 `rspfdisk-tui` 介面啟動。

這個媒體上的 GRUB2 只負責啟動 SPFDisk 環境，不會安裝到目標磁碟，也不能取代 Windows Boot Manager、Linux GRUB/systemd-boot 或 macOS boot components。

### 2. 選擇情境

先選取目標磁碟並檢查目前分割表，再選擇以下使用者情境：

| 情境 | 常見草稿 | 交接對象 |
|------|----------|----------|
| Windows | GPT/UEFI、ESP、MSR、Windows 目標，以及可選 Recovery 或 Data 空間 | 啟動 Windows Setup |
| Linux | ESP 或 BIOS boot partition、root、可選 home 與 swap | 啟動選定的 Linux 安裝程式 |
| macOS | GPT/GUID Partition Map 與 Apple APFS 目標分割區 | 啟動 macOS Installer 或使用 Disk Utility |
| Multiboot | 共用或指定 ESP，以及每個 OS 的獨立空間 | 使用各 OS 官方媒體逐一安裝 |

情境名稱描述預計結果。`windows_uefi_standard` 或 `macos_apfs_target` 等模板名稱是技術參考，不代表 SPFDisk 會安裝對應作業系統。

### 3. 預覽分割區草稿

任何寫入前都要檢查完整草稿，以及它和目前磁碟的差異：

- MBR 或 GPT，以及 UEFI/BIOS 模式。
- 每個分割區的名稱、類型、起始 sector、結束 sector、容量、對齊與檔案系統說明。
- 會保留的既有分割區。
- 新增、調整大小或移除的分割區。
- 是否沿用既有 EFI System Partition，或準備新的 ESP。

預覽階段可以調整容量、換情境或取消。預覽不應寫入目標。

### 4. 建立並確認備份

在確認前先完成備份：

1. 建立 `.rspbak` 版面備份。
2. 把副本存到目標磁碟以外的位置。
3. 檢查備份 metadata、磁碟身份、sector size 與分割區摘要。
4. 將備份和安裝筆記放在一起，之後可確認它們對應同一顆磁碟。

備份記錄的是分割區 metadata 與完整性資訊，不會複製個人檔案或作業系統資料。

### 5. Dry-run 並明確確認

在最後的 dry-run 與確認畫面，把預計情境和實際目標再次比對。只有以下條件全部成立才確認：

- 型號、容量與身份符合預計修改的磁碟。
- 已理解分割區差異，包括被移除或重新使用的空間。
- 備份已存在於目標磁碟以外。
- 開機模式與分割表符合 OS 安裝程式需求。
- 下一步所需的官方 OS 安裝媒體已準備好。

安全順序如下：

```text
Snapshot → Draft → Preview → Backup → Dry Run → Explicit Confirmation → Write / Rollback
```

真實磁碟寫入仍屬高風險。開發與自動化測試使用 image 檔。已開機的 live environment 不等於取得實體磁碟寫入許可；只有明確提供真實磁碟寫入、且通過安全門檻的 build 與流程才可執行。如果目前 TUI 或 image 流程拒絕實體磁碟，絕不可繞過限制。

### 6. 交接給 OS 安裝程式

通過許可的寫入完成後，重新開機或關機，從官方安裝程式開機，不是從 SPFDisk 媒體開機。可以保留 SPFDisk 媒體供檢查或復原使用，但不要期待目標磁碟此時已經能啟動作業系統。

#### Windows

啟動 Windows Setup，選取預計使用的 Windows 空間。Windows Setup 會安裝 Windows、完成所需檔案系統處理、建立或更新 Windows Boot Manager，並處理 Windows recovery 設定。這些安裝工作都不是 SPFDisk 的職責。

#### Linux

啟動該發行版安裝程式，依選定版面與發行版說明指定 ESP、root、home 與 swap。Linux 安裝程式會視需要格式化所選檔案系統，並安裝/設定 GRUB、systemd-boot 或該發行版選用的 bootloader。

#### macOS

在支援的 Apple 硬體上使用 macOS Installer 或 Disk Utility。SPFDisk 只準備 GPT 分割區類型與容量。**SPFDisk 不會格式化 APFS。** APFS container/volume 的格式化與建立，以及 macOS boot components 的安裝，都必須由 macOS 完成。SPFDisk 不會設定 Hackintosh/OpenCore、FileVault 或 macOS Recovery。

#### Multiboot

一次安裝一個作業系統。每次安裝前再次確認目標磁碟與分割區，保留其他 OS 區域，並遵循該安裝程式的 bootloader 指引。共用 ESP 只是準備好的儲存空間，本身不是 bootloader 安裝。

### 應停止的情況

遇到以下情況請取消並先調查：

- 無法明確辨識目標磁碟。
- 預覽結果與預計情境不同。
- 無法建立或在目標以外驗證備份。
- 缺少安裝媒體，或安裝媒體與選定 OS 不符。
- SPFDisk 被要求格式化 APFS。
- 工具要求繞過真實磁碟安全門檻。

版面細節請參考 [quick-layouts.md](quick-layouts.md)、[windows-layout.md](windows-layout.md)、[linux-layout.md](linux-layout.md)、[macos-layout.md](macos-layout.md) 與 [safety.md](safety.md)。
