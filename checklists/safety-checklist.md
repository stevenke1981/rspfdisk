# Safety Checklist

## 寫入安全機制

- [x] 預設唯讀（CLI 無 `--write` 則不寫入）。
- [x] `--dry-run` 可用（預覽變更不寫入）。
- [x] writer 需要 `WriteToken`（由 `confirm_write()` 產生）。
- [x] 寫入前自動備份（`backup-before-write` 強制）。
- [x] 寫入前顯示 diff（`build_diff_report()`）。
- [x] 寫入前輸入磁碟代號確認（`disk_confirmation_phrase()` + `--confirm`）。
- [x] 寫入後讀回驗證。
- [x] 測試不碰真實系統碟（全部在 image file 上執行）。
- [x] APFS 不直接格式化（僅建立 type GUID）。
- [x] NTFS 預設 no-format（可選擇不格式化）。
- [x] 雙系統預設沿用既有 ESP。

## 真實磁碟保護

- [x] `PathKind` 區分 image vs 真實裝置。
- [x] Linux 分割區節點（`/dev/sdb1`）禁止寫入。
- [x] 系統碟偵測（root device check）。
- [x] 系統碟寫入需 `--accept-system-disk-risk`。
- [x] 可卸除式磁碟有較低風險評級。
- [x] `DangerAssessment` 提供風險分級。

## 測試驗證

- [x] 38 個快速測試通過（含 safety、danger、confirmation）。
- [x] 5 個 slow GPT 8GiB image 測試通過。
- [x] cargo clippy -- -D warnings 通過。
- [x] cargo fmt --check 通過。
