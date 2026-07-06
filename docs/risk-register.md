# risk-register.md — 風險清單

| 風險 | 等級 | 說明 | 緩解策略 |
|---|---:|---|---|
| 誤寫系統碟 | 極高 | 可能造成資料毀損與無法開機 | 預設唯讀、WriteToken、磁碟代號確認、備份 |
| GPT CRC 寫錯 | 高 | OS 無法辨識分割表 | golden tests、讀回驗證 |
| APFS 支援不完整 | 高 | macOS 資料風險 | 第一版不格式化 APFS |
| NTFS 格式化風險 | 中高 | 寫入錯誤可能造成 Windows 不可用 | 第一版預設 no-format |
| 多 ESP 混亂 | 中 | 雙系統 boot entry 混亂 | 偵測既有 ESP，預設沿用 |
| TUI 操作誤解 | 中 | 使用者誤按寫入 | Preview + 確認文字 |
| UEFI app 開發複雜 | 中 | no_std 與 Block IO 複雜 | 第二階段處理 |
| 不同 sector size | 中 | 512e/4Kn 對齊錯誤 | SectorSize abstraction |
