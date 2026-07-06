# data-formats.md — 資料格式

## .rspbak

建議格式：tar-like 或 zip-like container。

```text
backup.rspbak
├─ manifest.json
├─ raw/first-2048-sectors.bin
├─ mbr/mbr.bin
├─ gpt/primary-header.bin
├─ gpt/backup-header.bin
├─ gpt/entries.bin
├─ summary/partitions.json
└─ checksums.sha256
```

## manifest.json

```json
{
  "format": "rspbak",
  "version": 1,
  "created_at": "2026-07-06T08:30:00+08:00",
  "tool": "rspfdisk 0.1.0",
  "disk": {
    "path": "/dev/nvme0n1",
    "model": "Example SSD",
    "serial": "XXXX",
    "size_bytes": 512110190592,
    "logical_sector_size": 512,
    "physical_sector_size": 4096
  },
  "partition_table": "gpt",
  "notes": []
}
```

## LayoutDraft JSON

```json
{
  "table": "gpt",
  "boot_mode": "uefi",
  "partitions": [
    {
      "name": "EFI System",
      "start_lba": 2048,
      "size_bytes": 536870912,
      "type": "esp",
      "filesystem": "fat32"
    }
  ]
}
```
