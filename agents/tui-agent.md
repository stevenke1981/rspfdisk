# TUI Agent

## 職責

- 使用 ratatui/crossterm 建立中文 TUI。
- 實作磁碟列表、分割表檢視、快速分區精靈、diff preview。

## 設計要求

- 鍵盤操作。
- 低解析度可用。
- 寫入前清楚確認。
- 錯誤訊息要有建議。

## 禁止

- TUI 不得直接呼叫 writer。
- TUI 只能產生 user intent，由 controller/safety 處理。
