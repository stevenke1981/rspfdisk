# safety.md — 安全設計

## 核心理念

磁碟分割工具的第一責任不是功能多，而是不要誤傷使用者資料。

## 預設唯讀

所有命令預設唯讀：

```bash
rspfdisk inspect /dev/sda
rspfdisk layout windows-standard /dev/sda --dry-run
```

只有明確加上 `--write` 才可能寫入。

## WriteToken

writer API 不接受 bool。

錯誤設計：

```rust
write_table(device, layout, true)
```

正確設計：

```rust
let token = safety.confirm(change_plan)?;
writer.write_table(device, layout, token)?;
```

`WriteToken` 必須包含：

- disk id。
- timestamp。
- backup id。
- confirmation phrase hash。
- change plan hash。

## 寫入確認

TUI 寫入前顯示：

```text
即將修改磁碟：/dev/nvme0n1
磁碟容量：512 GiB
序號：XXXX

變更：
+ 建立 GPT
+ 建立 ESP 512MiB
+ 建立 Windows 180GiB
+ 建立 Data 330GiB

已建立備份：backup-20260706-083000.rspbak

請輸入磁碟代號確認：nvme0n1
```

輸入錯誤不得寫入。

## 系統碟偵測

若偵測到以下情況，加強警告或拒絕：

- 目前 root filesystem 所在磁碟。
- Windows 系統碟。
- 已掛載中的分區。
- 有 BitLocker / FileVault / LUKS 痕跡。
- SMART 狀態異常。

## 備份策略

寫入前必須備份：

- raw first sectors。
- MBR。
- GPT primary header。
- GPT backup header。
- GPT partition entries。
- JSON summary。

## 回滾說明

工具應產生 rollback instruction：

```text
若寫入後開機失敗：
1. 使用同一支 USB 開機。
2. 選 Restore Backup。
3. 選 backup-xxxx.rspbak。
4. 確認磁碟 identity。
5. 執行還原。
```

## 測試防線

CI 禁止真實磁碟寫入。

可寫入測試只允許：

```text
*.img
loop device explicitly created by test harness
disposable USB with marker file
```
