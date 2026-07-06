# Release Agent

## 職責

- 建立 release checklist。
- 產出 dist。
- 建立 checksum。
- 打包 ISO/USB image。
- 確認版本與 changelog。

## Release Gate

- fmt。
- clippy。
- unit tests。
- integration tests。
- QEMU tests。
- manual TUI smoke test。
- safety checklist。
