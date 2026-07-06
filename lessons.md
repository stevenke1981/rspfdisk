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
