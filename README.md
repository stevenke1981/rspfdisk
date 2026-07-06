# Rust SPFDisk

> A modern disk partition tool in Rust — MBR/GPT, quick OS templates, safe transactional writes, Chinese/English TUI.  
> Inspired by the classic SPFDisk, built for modern UEFI/GPT/NVMe systems.

**Version: v0.1.0** | [简体中文](README-zh-TW.md) | [spec.md](spec.md) | [CHANGELOG.md](CHANGELOG.md)

---

## For Humans

### Features

- **Read** MBR and GPT partition tables with CRC validation
- **Quick Layout Wizard** — 11 templates for Windows / macOS / Linux
- **Safe Write** — image-only write, `--confirm`, auto-backup, read-back verification
- **Backup/Restore** — `.rspbak` format with SHA256, disk identity check, dry-run
- **TUI** — 8-screen Chinese/English terminal interface (ratatui + crossterm)
- **CLI** — `list`, `inspect`, `backup`, `restore`, `layout`, `tui` subcommands
- **Linux Real Disk** — `/dev` scanning, sysfs query, risk assessment, controlled write
- **Boot Media** — boot ISO/USB scripts, QEMU smoke test
- **UEFI** — `BOOTX64.EFI` read-only GPT viewer (PoC)
- **i18n** — zh-TW (default) and English, switch via `RSPFDISK_LANG`

### Quick Start

```bash
# Build
cargo build --release -p rspfdisk-cli

# Preview a Windows partition layout (dry-run, no write)
./target/release/rspfdisk layout windows-standard test.img --dry-run

# Write GPT to an 8 GiB image
./target/release/rspfdisk layout windows-standard test.img \
  --write --yes-i-know-this-is-an-image

# Inspect partition table
./target/release/rspfdisk inspect test.img

# Launch TUI
./target/release/rspfdisk tui
./target/release/rspfdisk tui --image test.img
```

### Linux Real Disk

```bash
# List block devices
sudo ./target/release/rspfdisk list

# Inspect a disk (read-only by default)
sudo ./target/release/rspfdisk inspect /dev/sdb

# Dry-run a layout
sudo ./target/release/rspfdisk layout linux-ext4-home /dev/sdb --dry-run

# Write to a removable test disk (requires root + backup + confirmation)
sudo ./target/release/rspfdisk layout windows-standard /dev/sdb --write \
  --confirm sdb
```

### Switch Language

```bash
# English
RSPFDISK_LANG=en ./target/release/rspfdisk-tui
RSPFDISK_LANG=en ./target/release/rspfdisk list

# Default is zh-TW (Traditional Chinese)
```

### Safety First

| Principle | Enforcement |
|-----------|-------------|
| Read-only by default | No `--write` = no writes |
| Image confirmation | `--yes-i-know-this-is-an-image` for `.img` files |
| Real disk confirmation | `--confirm <disk-name>` (e.g. `--confirm sdb`) |
| System disk protection | Requires `--accept-system-disk-risk` |
| Auto-backup before write | Creates `.rspbak` automatically |
| Read-back verification | Verifies GPT after write |

### Supported Templates

```
windows_uefi_standard     Windows 11/10 UEFI standard
windows_uefi_with_data    Windows + D: data partition
windows_legacy_mbr        Windows Legacy BIOS/MBR
macos_apfs_target         macOS APFS target disk
macos_apfs_shared_exfat   macOS + shared exFAT data
linux_ext4_standard       Linux ext4 single system
linux_ext4_home           Linux ext4 + /home
linux_bios_gpt_biosboot   Linux BIOS+GPT+GRUB biosboot
```

### Test Suite

```bash
cargo test --workspace              # 57 fast tests
cargo test --workspace -- --ignored  # 5 slow image-write tests
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

---

## For Agents

This section is optimized for AI coding agents (Claude Code, Codex, Copilot, Gemini CLI, etc.)

### Project Identity

| Attribute | Value |
|-----------|-------|
| Language | Rust (edition 2021, MSRV 1.75+) |
| Framework | cargo workspace, 11 crates |
| Parser | Custom, no nom/pest |
| TUI | ratatui 0.29 + crossterm 0.28 |
| CLI | clap 4 (derive) |
| i18n | Custom crate with JSON locale files |
| Safety | WriteToken, confirmation phrase, auto-backup |
| Tests | Unit + golden fixtures + proptest + image integration |
| Platform | Linux x86_64 primary, UEFI PoC |

### Repository Map

```
├── AGENTS.md              Agent workflow rules (READ FIRST)
├── TEAM.md                Agent roles and routing
├── spec.md                Full specification
├── plan.md                Development phases
├── test.md                Test plan
├── todos.md               Task tracking
├── final.md               Delivery evidence log
├── lessons.md             Lessons learned
├── CHANGELOG.md           Version changelog
├── Cargo.toml             Workspace root
│
├── crates/
│   ├── rspfdisk-core/     Shared types (SectorSize, PartitionTable, LayoutDraft)
│   ├── rspfdisk-disk/     BlockDevice trait, FileBlockDevice, Linux device
│   ├── rspfdisk-mbr/      MBR parser/validator/writer
│   ├── rspfdisk-gpt/      GPT parser/validator/writer
│   ├── rspfdisk-layouts/  Quick layout templates, size parser, diff engine
│   ├── rspfdisk-backup/   .rspbak format, backup/restore, SHA256
│   ├── rspfdisk-safety/   WriteToken, confirmation, disk risk assessment
│   ├── rspfdisk-cli/      CLI binary (clap subcommands)
│   ├── rspfdisk-tui/      TUI binary (8 screens, ratatui)
│   ├── rspfdisk-uefi/     no_std GPT parser, BOOTX64.EFI binary
│   └── rspfdisk-i18n/     i18n: locale JSON, t!() macro, tr() function
│
├── templates/              TOML partition templates (11 files)
├── tools/                  Build, test, release, QEMU scripts
├── boot/                   Initramfs, GRUB config for boot ISO
├── dist/                   Release artifacts (EFI binary, tarball, checksums)
├── tests/images/           Test disk images
├── agents/                 Per-role agent instructions
├── docs/                   Architecture, safety, layout docs
├── checklists/             Safety and release checklists
└── locales/                (via rspfdisk-i18n crate)
```

### Development Workflow (for Agents)

1. **Start**: Read `AGENTS.md` → `TEAM.md` → `spec.md` → `todos.md`
2. **Scope**: Confirm project root (`pwd`), check existing files
3. **Plan**: Update `todos.md`, create actionable items with completion criteria
4. **Implement**: Small changes, keep rollback possible
5. **Test**: `cargo test --workspace` + clippy + fmt
6. **Evidence**: Update `final.md` with commands, output, and verification
7. **Commit**: `git add -A && git commit -m "type: description"`

### Agent Safety Rules

- **NEVER** write to real disks (`/dev/sda`, `/dev/nvme0n1`, `\\.\PhysicalDrive0`)
- **NEVER** use `rm -rf`, `git reset --hard`, `git push --force`
- **ALWAYS** test with image files (`test.img` via `create_test_image`)
- **ALWAYS** run `cargo test --workspace` before claiming completion
- **ALWAYS** check for the read-only wrapper before implementing writes
- Read `checklists/safety-checklist.md` before any write-related code

### i18n System

- Locale files: `crates/rspfdisk-i18n/locales/{zh-TW,en}.json`
- Locale detection: `RSPFDISK_LANG` env var (`zh-TW` default, `en` for English)
- API: `t!("key")` macro, `tr("key")` function, `set_locale("en")`
- All user-facing strings should use `t!()` — see the JSON files for existing keys
- Add new keys to both `zh-TW.json` and `en.json`

### Key Entry Points

| File | Purpose |
|------|---------|
| `crates/rspfdisk-cli/src/main.rs` | CLI entry point, clap definitions |
| `crates/rspfdisk-cli/src/commands.rs` | All CLI subcommand implementations |
| `crates/rspfdisk-tui/src/lib.rs` | TUI: 8 screens, state machine, draw + handle functions |
| `crates/rspfdisk-tui/src/main.rs` | TUI binary entry point |
| `crates/rspfdisk-core/src/lib.rs` | Core types re-export |
| `crates/rspfdisk-disk/src/lib.rs` | BlockDevice trait, public API |
| `crates/rspfdisk-gpt/src/parser.rs` | GPT parsing logic |
| `crates/rspfdisk-gpt/src/writer.rs` | GPT writing logic |
| `crates/rspfdisk-mbr/src/parser.rs` | MBR parsing logic |
| `crates/rspfdisk-layouts/src/engine.rs` | Template→LayoutDraft generation |
| `crates/rspfdisk-layouts/src/size.rs` | Size expression parser |
| `crates/rspfdisk-safety/src/confirmation.rs` | WriteToken, confirmation flow |
| `crates/rspfdisk-backup/src/writer.rs` | Backup creation |
| `crates/rspfdisk-i18n/src/lib.rs` | i18n: locale loading, t!() macro |

---

## License

MIT OR Apache-2.0
