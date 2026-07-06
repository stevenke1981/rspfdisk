# quick-layouts.md — 快速分區精靈規格

## 目的

快速分區精靈讓使用者不用手動計算 ESP、MSR、Root、Home、Swap、Recovery 等分區大小。工具根據 OS 模板產生分割草稿，讓使用者可預覽、調整、備份後再寫入。

## 使用流程

```text
1. 選擇目標磁碟
2. 選擇 OS：Windows / macOS / Linux
3. 選擇使用場景
4. 輸入容量偏好
5. 產生分區草稿
6. 顯示差異
7. 自動建立備份
8. 使用者確認
9. 寫入
10. 重新讀回驗證
```

## Template DSL

模板使用 TOML。

### 欄位

```toml
name = "windows_uefi_standard"
display_name = "Windows 11 / 10 UEFI 標準分區"
table = "gpt"
boot_mode = "uefi"
min_disk_size = "64GiB"
```

### Partition 欄位

```toml
[[partitions]]
name = "EFI System"
size = "512MiB"
type = "esp"
filesystem = "fat32"
flags = ["boot", "esp"]
mount = "/boot/efi"
note = "optional note"
```

## Size 表達式

支援：

```text
512MiB
1GiB
80GiB
fill
fill-minus:1GiB
auto:swap
percent:50
minmax:40GiB:120GiB
```

### auto:swap

初版建議：

```text
RAM <= 4GiB    swap = RAM
RAM <= 16GiB   swap = 8GiB
RAM <= 64GiB   swap = 16GiB
RAM > 64GiB    swap = 32GiB
```

若無法偵測 RAM，預設 8GiB。

## 對齊規則

- 預設 1MiB 對齊。
- 不使用 CHS。
- 4K physical sector 必須檢查。
- 小於最小磁碟容量時拒絕產生草稿。

## 雙系統策略

若偵測到既有 ESP：

```text
[建議] 沿用既有 ESP
[風險] 新增第二個 ESP 可能造成開機項目混亂
```

工具應提供：

```text
- 沿用既有 ESP
- 建立新 ESP
- 手動選擇 ESP
- 取消
```

## 格式化策略

第一版：

- FAT32：可支援。
- ext4：可支援或透過外部 mkfs.ext4。
- swap：可支援或透過 mkswap。
- NTFS：可選，建議第一版預設 no-format。
- exFAT：可選。
- APFS：不直接格式化。

## 草稿範例

```text
Template: Windows + D 槽
Disk: 512 GiB GPT

+ ESP       512 MiB
+ MSR       16 MiB
+ Windows   180 GiB
+ Recovery  1 GiB
+ Data      330 GiB

No write performed.
```
