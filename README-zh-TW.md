# Rust SPFDisk

> 現代磁碟分割工具 — Rust 實作，支援 MBR/GPT、快速分區模板、安全交易式寫入、中英文 TUI。  
> 繼承經典 SPFDisk 精神，專為現代 UEFI/GPT/NVMe 系統打造。

**版本：v0.1.0** | [English](README.md) | [可開機導引安裝](docs/bootable-guided-install.md) | [spec.md](spec.md) | [CHANGELOG.md](CHANGELOG.md)

---

## 給人類使用者

### 從這裡開始：可開機導引安裝

SPFDisk 是**分割區準備工具**，不是作業系統安裝程式。它可以檢查磁碟、建立 Windows/Linux/macOS/多重開機分割區草稿、顯示變更、建立備份，並在明確確認後套用通過安全門檻的配置。

它**不會安裝 Windows、Linux 或 macOS**，也不會安裝 Windows Boot Manager、Linux GRUB/systemd-boot、macOS boot components 或其他目標磁碟 bootloader。SPFDisk 媒體上的 GRUB2 只負責啟動 SPFDisk 環境，不會被安裝到目標磁碟。準備完成後，請改用對應的官方 OS 安裝媒體，讓該安裝程式完成 OS 與 bootloader 工作。

完整的新手流程請參考[可開機導引安裝](docs/bootable-guided-install.md)。

### 導引流程

1. **從 GRUB2 媒體開機。** 依[開機媒體文件](docs/boot-media.md)建立或取得 SPFDisk ISO/USB。在韌體開機選單選取該媒體，再從 GRUB2 選擇 Rust SPFDisk 導引/TUI 入口。CLI shell 入口只供復原或進階使用。
2. **選取並檢查目標。** 確認磁碟型號、容量、序號或其他身份資訊。選擇配置前先讀取目前的 MBR/GPT 分割表。練習與自動化測試請使用 image 檔。
3. **選擇情境。** 選取 **Windows**、**Linux**、**macOS** 或 **Multiboot**，再選擇對應版面與容量選項。情境名稱是給使用者的選擇；模板識別名稱屬於實作細節。
4. **預覽分割區草稿。** 檢查分割表類型、起訖 sector、容量、對齊、檔案系統/類型說明，以及相對於目前磁碟的差異。預覽期間不應發生寫入。
5. **寫入前備份。** 建立 `.rspbak` 備份，並把副本放在目標磁碟以外。檢查備份 metadata 與磁碟身份。
6. **Dry-run 並明確確認。** 再次核對目標與最後差異。只有明確確認步驟可以授權通過安全門檻的寫入。磁碟身份、版面或預計安裝的 OS 只要有不清楚，就取消。
7. **交接給 OS 安裝程式。** 目標準備完成後重新開機或關機，改從官方 Windows、Linux 或 macOS 安裝媒體開機，選取預計使用的已準備空間並依安裝程式操作。多重開機時逐一安裝各 OS，且每次都確認目標分割區。

### 情境交接

| 情境 | SPFDisk 準備內容 | OS 安裝程式完成內容 |
|------|------------------|---------------------|
| Windows | 通常是 GPT/UEFI，包含 ESP、MSR、Windows 目標，以及可選的 Recovery/Data 空間 | Windows 檔案、依需求完成檔案系統設定、Windows Boot Manager 與復原設定 |
| Linux | 依選定版面建立 ESP 或 BIOS boot partition，以及 root、home、swap 空間 | Linux 檔案、依需求完成檔案系統設定，以及 GRUB/systemd-boot 安裝與設定 |
| macOS | GPT/GUID Partition Map、Apple APFS 類型的目標分割區，以及可選共用空間 | macOS 安裝、APFS 格式化、volume、Recovery 與 Apple boot components |
| Multiboot | 共用或指定的 ESP 空間，以及各 OS 的獨立區域 | 每個官方安裝程式各自安裝 OS 與 bootloader；準備共用 ESP 本身不會安裝 bootloader |

**macOS 邊界：** SPFDisk 不會格式化 APFS。APFS 必須由 macOS 安裝程式或 Disk Utility 格式化與建立。SPFDisk 也不會設定 Hackintosh/OpenCore、FileVault 或 macOS Recovery。

### 安全要求

每次版面變更都遵循以下順序：

```text
Snapshot → Draft → Preview → Backup → Dry Run → Explicit Confirmation → Write / Rollback
```

- 預設是唯讀或只預覽。備份與預覽都不能讓選錯目標變得安全。
- 開發與自動化測試使用 image 檔。絕不可把真實系統碟當成測試 image。
- 真實磁碟寫入仍屬高風險，必須通過適用的安全門檻、確認正確磁碟身份、把備份存到其他位置，並明確確認。不要只依 `/dev/sdX` 或磁碟代號選擇目標。
- 進行真實磁碟操作前，視情況關閉應用程式並卸載分割區。現有版面或目標身份無法明確判定時，立即停止。
- 不要因為 SPFDisk 寫入成功，就假設 OS 或 bootloader 已安裝。最後的 boot files 由對應 OS 安裝程式提供。
- APFS 目標由 SPFDisk 準備，但不由 SPFDisk 格式化。

### CLI 與 Image 快速開始

CLI 適合用來做可重複的 image 預覽與測試：

```bash
# 建置
cargo build --release -p rspfdisk-cli

# 預覽 Windows 分割區版面（dry-run，不寫入）
./target/release/rspfdisk layout windows-standard test.img --dry-run

# 只有在明確完成 image 確認後，才寫入 image 的 GPT
./target/release/rspfdisk layout windows-standard test.img \
  --write --yes-i-know-this-is-an-image

# 檢查完成後的分割表
./target/release/rspfdisk inspect test.img

# 啟動 TUI
./target/release/rspfdisk tui
./target/release/rspfdisk tui --image test.img
```

### 延伸閱讀

- [可開機導引安裝](docs/bootable-guided-install.md) — 新手 checklist 與 OS 交接
- [開機媒體](docs/boot-media.md) — ISO、USB 與 QEMU 開機路徑
- [快速分區](docs/quick-layouts.md) — 草稿與對齊規則
- [Windows 版面](docs/windows-layout.md)、[Linux 版面](docs/linux-layout.md) 與 [macOS 版面](docs/macos-layout.md) — 各情境細節
- [安全性](docs/safety.md) — 寫入與復原限制

### 測試套件

```bash
cargo test --workspace               # 快速測試
cargo test --workspace -- --ignored  # 慢速 image 寫入測試
cargo clippy --workspace -- -D warnings
cargo fmt --check
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
