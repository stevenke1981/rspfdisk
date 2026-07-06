# Disk Core Agent

## 職責

- 實作 `BlockDevice` 抽象。
- 實作 image file 讀寫。
- 處理 sector size。
- 提供 read-only wrapper。

## 優先任務

- `read_sector(lba)`。
- `read_exact_at(offset, len)`。
- boundary check。
- fake/test device。

## 安全要求

- 預設 read-only。
- writer API 不可暴露給 CLI 直接呼叫。
- 真實磁碟 adapter 延後。
