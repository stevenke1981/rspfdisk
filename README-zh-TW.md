# Rust SPFDisk

> 現代磁碟分割工具 — Rust 實作，支援 MBR/GPT、快速分區模板、安全交易式寫入、中英文 TUI。  
> 繼承經典 SPFDisk 精神，專為現代 UEFI/GPT/NVMe 系統打造。

**版本：v0.1.0** | [English](README.md) | [spec.md](spec.md) | [CHANGELOG.md](CHANGELOG.md)

---

## 給人類使用者

### 功能特色

- **讀取** MBR 與 GPT 分割表（含 CRC 驗證）
- **快速分區精靈** — 11 組 Windows / macOS / Linux 模板
- **安全寫入** — 僅限 image 寫入、`--confirm` 確認、自動備份、讀回驗證
- **備份/還原** — `.rspbak` 格式（SHA256、磁碟身份檢查、dry-run）
- **TUI 介面** — 8 螢幕中英文終端介面（ratatui + crossterm）
- **CLI 指令** — `list`、`inspect`、`backup`、`restore`、`layout`、`tui`
- **Linux 真實磁碟** — `/dev` 掃描、sysfs 查詢、風險評估、受控寫入
- **開機媒體** — Boot ISO/USB 打包腳本、QEMU 測試
- **UEFI** — `BOOTX64.EFI` 唯讀 GPT 檢視器（概念驗證）
- **多國語系** — 繁體中文（預設）與英文，透過 `RSPFDISK_LANG` 切換

### 快速開始

```bash
# 建置
cargo build --release -p rspfdisk-cli

# 預覽 Windows 標準分區（dry-run，不寫入）
./target/release/rspfdisk layout windows-standard test.img --dry-run

# 寫入 GPT 到 8 GiB image
./target/release/rspfdisk layout windows-standard test.img \
  --write --yes-i-know-this-is-an-image

# 檢視分割表
./target/release/rspfdisk inspect test.img

# 啟動 TUI
./target/release/rspfdisk tui
./target/release/rspfdisk tui --image test.img
```

### Linux 真實磁碟

```bash
# 列出區塊裝置
sudo ./target/release/rspfdisk list

# 檢視磁碟（預設唯讀）
sudo ./target/release/rspfdisk inspect /dev/sdb

# Dry-run 預覽分區
sudo ./target/release/rspfdisk layout linux-ext4-home /dev/sdb --dry-run

# 寫入可卸除測試碟（需 root + 備份 + 確認）
sudo ./target/release/rspfdisk layout windows-standard /dev/sdb --write \
  --confirm sdb
```

### 切換語系

```bash
# 英文
RSPFDISK_LANG=en ./target/release/rspfdisk-tui

# 繁體中文（預設，可不設定）
RSPFDISK_LANG=zh-TW ./target/release/rspfdisk-tui
```

### 安全原則

| 原則 | 強制方式 |
|------|----------|
| 預設唯讀 | 沒加 `--write` 就不會寫入 |
| Image 確認 | `--yes-i-know-this-is-an-image` |
| 真實磁碟確認 | `--confirm <磁碟代號>`（如 `--confirm sdb`） |
| 系統碟保護 | 需加 `--accept-system-disk-risk` |
| 寫入前自動備份 | 自動建立 `.rspbak` |
| 寫入後驗證 | 寫入 GPT 後重新讀回驗證 |

### 支援的模板

```
windows_uefi_standard     Windows 11/10 UEFI 標準分區
windows_uefi_with_data    Windows + D 槽資料分區
windows_legacy_mbr        Windows Legacy BIOS/MBR
macos_apfs_target         macOS APFS 目標碟
macos_apfs_shared_exfat   macOS + 共用 exFAT 資料
linux_ext4_standard       Linux ext4 單系統
linux_ext4_home           Linux ext4 + 獨立 /home
linux_bios_gpt_biosboot   Linux BIOS+GPT+GRUB 開機
```

### 測試

```bash
cargo test --workspace              # 57 個快速測試
cargo test --workspace -- --ignored  # 5 個慢速 image 寫入測試
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### TUI 操作指南

啟動 TUI 後，8 個螢幕的操作流程：

```
主選單 → [1] 磁碟列表 → [Enter] 分割表 → [F] 快速分區 → [Enter] 預覽
                                                              ↓
                                                        [E] 容量編輯器
                                                        [B] 備份確認
                                                        [W] 寫入確認
```

---

## 給 AI Agent

本節專為 AI 程式設計助手（Claude Code、Codex、Copilot、Gemini CLI 等）優化。

### 專案身份

| 屬性 | 值 |
|------|-----|
| 語言 | Rust（edition 2021，MSRV 1.75+） |
| 框架 | cargo workspace，11 個 crate |
| 解析器 | 自製，無 nom/pest 依賴 |
| TUI | ratatui 0.29 + crossterm 0.28 |
| CLI | clap 4（derive 模式） |
| i18n | 自製 crate，JSON 語系檔 |
| 安全機制 | WriteToken、確認詞、自動備份 |
| 測試 | 單元測試 + golden fixtures + proptest + image 整合測試 |
| 平台 | Linux x86_64 為主，UEFI PoC |

### 目錄結構

```
├── AGENTS.md              Agent 工作規範（優先閱讀）
├── TEAM.md                Agent 角色與分流
├── spec.md                完整需求規格
├── plan.md                開發階段計畫
├── test.md                測試計畫
├── todos.md               任務追蹤
├── final.md               交付驗收證據
├── lessons.md             經驗教訓
├── CHANGELOG.md           版本變更紀錄
├── Cargo.toml             Workspace 根目錄
│
├── crates/
│   ├── rspfdisk-core/     共用型別（SectorSize, PartitionTable, LayoutDraft）
│   ├── rspfdisk-disk/     BlockDevice trait, FileBlockDevice, Linux 裝置
│   ├── rspfdisk-mbr/      MBR 解析器/驗證器/寫入器
│   ├── rspfdisk-gpt/      GPT 解析器/驗證器/寫入器
│   ├── rspfdisk-layouts/  快速分區模板、大小解析器、差異引擎
│   ├── rspfdisk-backup/   .rspbak 格式、備份/還原、SHA256
│   ├── rspfdisk-safety/   WriteToken、確認流程、磁碟風險評估
│   ├── rspfdisk-cli/      CLI 二進位檔（clap 子指令）
│   ├── rspfdisk-tui/      TUI 二進位檔（8 螢幕、ratatui）
│   ├── rspfdisk-uefi/     no_std GPT 解析器、BOOTX64.EFI
│   └── rspfdisk-i18n/     i18n：語系 JSON、t!() 巨集、tr() 函式
│
├── templates/              TOML 分區模板（11 個檔案）
├── tools/                  建置、測試、釋出、QEMU 腳本
├── boot/                   開機 initramfs、GRUB 設定
├── dist/                   釋出產物（EFI、tarball、checksums）
├── tests/images/           測試用磁碟映像檔
├── agents/                 各角色 Agent 說明
├── docs/                   架構、安全、分區模板文件
└── checklists/             安全與釋出檢查清單
```

### Agent 開發流程

1. **開始**：閱讀 `AGENTS.md` → `TEAM.md` → `spec.md` → `todos.md`
2. **範圍確認**：確認專案根目錄（`pwd`），檢查必要文件是否存在
3. **規劃**：更新 `todos.md`，建立可驗證的任務項目
4. **實作**：小步修改，確保可回溯
5. **測試**：`cargo test --workspace` + clippy + fmt
6. **證據**：更新 `final.md`，記錄指令、輸出與驗證結果
7. **提交**：`git add -A && git commit -m "type: description"`

### Agent 安全規則

- **嚴禁**寫入真實磁碟（`/dev/sda`、`/dev/nvme0n1`、`\\.\PhysicalDrive0`）
- **嚴禁**使用 `rm -rf`、`git reset --hard`、`git push --force`
- **必須**使用 image 檔測試（透過 `create_test_image` 建立）
- **必須**在宣稱完成前執行 `cargo test --workspace`
- **必須**在實作寫入功能前檢查唯讀包裝器
- 寫入相關程式碼前請先閱讀 `checklists/safety-checklist.md`

### 多國語系系統

- 語系檔案：`crates/rspfdisk-i18n/locales/{zh-TW,en}.json`
- 語系偵測：`RSPFDISK_LANG` 環境變數（`zh-TW` 預設、`en` 英文）
- API：`t!("key")` 巨集、`tr("key")` 函式、`set_locale("en")`
- 所有使用者可見字串都應使用 `t!()` — 請參考 JSON 檔現有鍵值
- 新增鍵值時必須同時加入 `zh-TW.json` 和 `en.json`

### 關鍵進入點

| 檔案 | 用途 |
|------|------|
| `crates/rspfdisk-cli/src/main.rs` | CLI 進入點、clap 定義 |
| `crates/rspfdisk-cli/src/commands.rs` | 所有 CLI 子指令實作 |
| `crates/rspfdisk-tui/src/lib.rs` | TUI：8 螢幕、狀態機、繪製 + 按鍵處理 |
| `crates/rspfdisk-tui/src/main.rs` | TUI 二進位檔進入點 |
| `crates/rspfdisk-core/src/lib.rs` | 核心型別重新匯出 |
| `crates/rspfdisk-disk/src/lib.rs` | BlockDevice trait、公開 API |
| `crates/rspfdisk-gpt/src/parser.rs` | GPT 解析邏輯 |
| `crates/rspfdisk-gpt/src/writer.rs` | GPT 寫入邏輯 |
| `crates/rspfdisk-mbr/src/parser.rs` | MBR 解析邏輯 |
| `crates/rspfdisk-layouts/src/engine.rs` | 模板 → LayoutDraft 產生 |
| `crates/rspfdisk-layouts/src/size.rs` | 大小表達式解析器 |
| `crates/rspfdisk-safety/src/confirmation.rs` | WriteToken、確認流程 |
| `crates/rspfdisk-backup/src/writer.rs` | 備份建立 |
| `crates/rspfdisk-i18n/src/lib.rs` | i18n：語系載入、t!() 巨集 |

---

## 授權

MIT OR Apache-2.0
