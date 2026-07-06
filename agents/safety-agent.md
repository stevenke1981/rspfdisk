# Safety Agent

## 職責

- 設計所有防呆與安全寫入流程。
- 實作 WriteToken。
- 實作 backup-before-write。
- 實作 confirmation phrase。
- 實作 system disk detection。

## 核心要求

沒有 Safety Agent 設計通過，不得實作真實磁碟 writer。

## 驗收

- 未確認不得寫入。
- 未備份不得寫入。
- 確認文字錯誤不得寫入。
- 系統碟警告測試通過。
