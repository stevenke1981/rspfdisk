# AGENTS.md — Rust SPFDisk 開發總規範

## 專案目標

建立一套以 Rust 開發的現代版 SPFDisk 類工具，具備：

1. 開機碟可用。
2. MBR / GPT 分割表管理。
3. Windows / macOS / Linux 快速分區模板。
4. 中文 TUI 操作介面。
5. 安全備份、預覽、交易式寫入、測試優先。
6. 最終可輸出 CLI、TUI、Linux boot ISO、USB image，第二階段輸出 UEFI app。

## 絕對禁止

任何 agent 不得：

- 在未明確確認前對真實磁碟寫入。
- 使用 `rm -rf`、`del /s`、`rmdir /s`、`git reset --hard`、`git push --force`、格式化磁碟、覆寫分割表等破壞性命令，除非使用者或上層 Orchestrator 明確批准。
- 把本專案檔案寫到其他專案目錄。
- 把不同專案的 `plan.md`、`todos.md`、`spec.md` 混在一起。
- 在沒有測試 image 的情況下實作真實磁碟寫入。
- 宣稱功能已完成但沒有測試紀錄。

## 專案邊界

所有開發都必須限制在本專案根目錄內。

```text
PROJECT_ROOT = rust-spfdisk/
```

每次開始工作前，agent 必須檢查：

```text
pwd / Get-Location
ls / dir
存在 AGENTS.md、TEAM.md、spec.md、plan.md、todos.md、test.md
```

若目前路徑不是專案根目錄，必須停止並回報。

## Controlled Workflow

每個任務都要遵守：

```text
1. Read
   讀 AGENTS.md / TEAM.md / spec.md / todos.md / test.md。

2. Plan
   只修改與任務相關的檔案，列出預期變更。

3. Implement
   小步提交，避免大爆改。

4. Test
   先單元測試，再 image 測試，再 QEMU boot 測試。

5. Evidence
   回報測試指令、輸出摘要、失敗原因與修正。

6. Update docs
   更新 todos.md / final.md / lessons.md。
```

## 模型分工建議

- 架構與多步驟設計：GPT-5.5 high。
- 多次失敗、跨模組架構重設、測試反覆不過：GPT-5.5 xhigh。
- 小修、文件整理、todos 更新：GPT-5.4-Mini medium。
- 一般 bug fix、refactor、crate 內功能：GPT-5.4 high。

## Agents 角色

詳見 `agents/` 目錄：

- `orchestrator.md`
- `system-architect.md`
- `disk-core-agent.md`
- `mbr-gpt-agent.md`
- `quick-layout-agent.md`
- `tui-agent.md`
- `boot-media-agent.md`
- `safety-agent.md`
- `tester-agent.md`
- `reviewer-agent.md`
- `docs-agent.md`
- `release-agent.md`

## 寫入磁碟安全規則

所有磁碟變更必須先形成 `ChangePlan`：

```text
現況 Snapshot
↓
分區草稿 Draft Layout
↓
Diff Preview
↓
Backup Plan
↓
Dry Run
↓
Explicit Confirmation
↓
Atomic Write / Rollback
```

沒有 `ChangePlan`，不得呼叫 writer。

## 交付標準

功能算完成必須同時具備：

- 程式碼完成。
- 單元測試完成。
- 測試 image 完成。
- 至少一個成功驗收案例。
- 文件更新。
- `final.md` 有證據紀錄。
