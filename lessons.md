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

## Lesson #5 — 2026-07-09
**Trigger:** GPT writer 驗證發現 Windows 標準模板草稿尾端超出 GPT usable range
**Rule:** writer 必須把 `LayoutDraft` 視為寫入契約，寫入前驗證 table type、LBA range、overlap 與 sector alignment；layout engine 計算 `fill` / `fill-minus` 時必須扣除後續分割區與 1MiB 對齊 gap
**Source:** rust-spfdisk GPT writer/layout hardening

## Lesson #6 — 2026-07-09
**Trigger:** CLI integration tests 覆寫 tracked `.rspbak` fixture
**Rule:** 測試產生的 image/backup 檔應放入 ignored generated 目錄，避免 `cargo test` 污染 git working tree
**Source:** rust-spfdisk workspace verification

## Lesson #7 — 2026-07-10
**Trigger:** TUI 在確認詞正確後顯示「寫入完成（模擬）」但沒有寫入 image
**Rule:** 破壞性操作的 UI 不得宣稱模擬結果為成功；寫入流程必須要求有效備份、限制目標類型，並以讀回驗證作為完成條件
**Source:** rust-spfdisk TUI image write hardening
