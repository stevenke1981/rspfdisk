# lessons.md — 經驗記錄

## 記錄格式

```text
日期：
類型：成功 / 失敗 / 風險
任務：
現象：
原因：
處理：
下次避免方式：
測試證據：
```

## 初始經驗

日期：2026-07-06  
類型：風險  
任務：Rust SPFDisk 需求規劃  
現象：磁碟分割工具若直接寫入真實磁碟，風險極高。  
處理：規格中強制預設唯讀、dry-run、WriteToken、自動備份、確認文字與 image-only 測試。  
下次避免方式：任何 writer 任務都先交給 Safety Agent review。

---

## Lesson #2 — 2026-07-06
**Trigger:** 專案初始化第一次 commit 前發現大量 formatting 問題
**Rule:** 在 git commit 前先執行 `cargo fmt --check` 和 `cargo clippy --workspace -- -D warnings`，確保程式碼品質
**Source:** rust-spfdisk 全面驗收

## Lesson #3 — 2026-07-06
**Trigger:** 8GiB 測試 image 檔案不小心理進入 git stage
**Rule:** 在 `git init` 前先建立 `.gitignore`，排除大型測試產物（`tests/images/*.img`）
**Source:** rust-spfdisk 全面驗收

## Lesson #4 — 2026-07-06
**Trigger:** 第一次 git commit 前 `gitPrecommitReview` 顯示 214 個檔案
**Rule:** 在 commit 前檢查 `# Staged changes summary`，確認沒有意外包含大型 binary 或 IDE 設定檔
**Source:** rust-spfdisk 全面驗收
