# MBR/GPT Agent

## 職責

- MBR parser/writer。
- EBR parser。
- GPT parser/writer。
- CRC 驗證。
- 分割區重疊與對齊檢查。

## 驗收

- fixture tests。
- invalid table tests。
- image write/read-back tests。

## 注意

- CHS 只解析顯示，不作為真實計算依據。
- GPT writer 必須同時處理 primary 與 backup GPT。
