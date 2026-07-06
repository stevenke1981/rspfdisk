# Rust SPFDisk

現代版 SPFDisk 類磁碟分割工具 — Rust 實作，支援 MBR/GPT、快速分區模板、安全備份與中文 TUI。

**版本：v0.1.0**

## 功能

- 讀取 MBR/GPT 分割表（含 CRC 驗證）
- Windows / macOS / Linux 快速分區模板（dry-run 預覽）
- 對 image file 安全寫入 GPT（需明確確認）
- `.rspbak` 備份格式
- Linux `/dev` 磁碟列表與唯讀檢視
- 中文 TUI（`rspfdisk-tui`）
- Boot ISO/USB 打包腳本
- UEFI 唯讀 GPT viewer（`BOOTX64.EFI` PoC）

## 建置

```bash
# 需要 Rust 1.75+
cargo build --release -p rspfdisk-cli
```

產出：`target/release/rspfdisk`

## 快速開始（image file）

```bash
# 預覽 Windows 標準分區（不寫入）
cargo run -p rspfdisk-cli -- layout windows-standard test.img --dry-run

# 寫入 8GiB image（僅限 .img）
cargo run -p rspfdisk-cli -- layout windows-standard test.img \
  --write --yes-i-know-this-is-an-image

# 檢視分割表
cargo run -p rspfdisk-cli -- inspect test.img --json
```

## Linux 真實磁碟（唯讀預設）

```bash
rspfdisk list
rspfdisk inspect /dev/nvme0n1 --json
rspfdisk layout windows-standard /dev/sdb --dry-run

# 寫入可卸除測試碟（需 root + 備份 + 確認）
sudo rspfdisk layout windows-standard /dev/sdb --write \
  --confirm sdb
```

## Boot media

```bash
# Linux/WSL
bash tools/make-boot-iso.sh
bash tools/qemu-test.sh --validate-only

# UEFI app
bash tools/build-uefi.sh
bash tools/qemu-uefi-app.sh
```

## 測試

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
bash tools/make-release.sh --verify-only   # Linux
pwsh tools/make-release.ps1 -VerifyOnly    # Windows

# Linux 完整驗收（含 QEMU UEFI smoke）
bash tools/verify-linux.sh --with-qemu
bash tools/verify-linux.sh --with-iso --with-qemu
```

## 安全原則

- **預設唯讀** — 沒有 `--write` 不會寫入
- **image 確認** — `--yes-i-know-this-is-an-image`
- **真實磁碟** — `--confirm <磁碟代號>` + 自動備份 + root
- **系統碟** — 需 `--accept-system-disk-risk`
- CI 與測試禁止寫入真實系統碟

## 專案結構

```text
crates/
  rspfdisk-core      共用型別
  rspfdisk-disk      BlockDevice 抽象
  rspfdisk-mbr/gpt   分割表 parser/writer
  rspfdisk-layouts   快速分區模板
  rspfdisk-safety    WriteToken 安全層
  rspfdisk-cli       rspfdisk 命令列
  rspfdisk-tui       中文 TUI
  rspfdisk-uefi      UEFI GPT viewer PoC
templates/           分區模板 TOML
tools/               打包與測試腳本
```

## 文件

- [spec.md](spec.md) — 需求規格
- [plan.md](plan.md) — 開發計畫
- [test.md](test.md) — 測試計畫
- [final.md](final.md) — 驗收紀錄
- [CHANGELOG.md](CHANGELOG.md) — 版本變更

## 授權

MIT OR Apache-2.0