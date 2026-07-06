# TEAM.md — 共享開發規範

## 團隊語言

- 使用繁體中文回報。
- 程式碼註解可中英混用，但公開 API 文件建議英文。
- 錯誤訊息與 TUI 文案第一版提供繁中，第二階段加英文語系。

## 工作模式

每個 agent 只做自己負責的區域，不跨區亂改。

```text
Orchestrator      負責任務拆分與驗收
Architect         負責 crate 架構與介面
Disk Core         負責 sector/device/image 抽象
MBR/GPT           負責分割表 parser/writer
Quick Layout      負責 Windows/macOS/Linux 快速模板
TUI               負責中文操作介面
Boot Media        負責 ISO/USB/QEMU
Safety            負責防呆、備份、回滾
Tester            負責測試矩陣與證據
Reviewer          負責 code review
Docs              負責文件
Release           負責打包與版本
```

## Git 規範

建議 branch：

```text
main
feat/disk-core
feat/mbr-gpt
feat/quick-layouts
feat/tui
feat/boot-media
feat/safety
feat/tests
```

commit message：

```text
feat(core): add sector device trait
fix(gpt): validate backup header CRC
test(layout): add windows dual-boot template fixture
docs(safety): document destructive command policy
```

## 不混專案規則

每個專案都必須有自己的：

```text
AGENTS.md
TEAM.md
spec.md
plan.md
todos.md
test.md
final.md
```

不得把其他專案的 todos 或 plan 複製進本專案，除非明確標示為參考。

## lessons.md 規範

每次遇到錯誤，必須記錄：

```text
日期：
錯誤：
原因：
修正：
避免方式：
測試證據：
```

成功經驗也要記錄，讓下一次任務先參考成功路線。
