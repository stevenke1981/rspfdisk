# Rust SPFDisk

> A modern disk partition tool in Rust — MBR/GPT, quick OS templates, safe transactional writes, Chinese/English TUI.  
> Inspired by the classic SPFDisk, built for modern UEFI/GPT/NVMe systems.

**Version: v0.1.0** | [繁體中文](README-zh-TW.md) | [Bootable Guided Install](docs/bootable-guided-install.md) | [spec.md](spec.md) | [CHANGELOG.md](CHANGELOG.md)

---

## For Humans

### Start Here: Bootable Guided Install

SPFDisk is the **partition-preparation step**, not an operating system installer. It can inspect a disk, build a Windows/Linux/macOS/multiboot partition draft, show the changes, make a backup, and apply a permitted layout after explicit confirmation.

It does **not** install Windows, Linux, or macOS. It also does not install Windows Boot Manager, Linux GRUB/systemd-boot, macOS boot components, or any other target-disk bootloader. GRUB2 on the SPFDisk media only boots the SPFDisk environment. After preparation, boot the appropriate official OS installer and let that installer complete the OS and bootloader work.

For the complete newcomer walkthrough, see [Bootable Guided Install](docs/bootable-guided-install.md).

### Guided Workflow

1. **Boot the GRUB2 media.** Create or obtain the SPFDisk ISO/USB described in [boot media](docs/boot-media.md). In the firmware boot menu, select that media and choose the Rust SPFDisk guided/TUI entry from GRUB2. Use the CLI shell entry only for recovery or advanced work.
2. **Select and inspect the target.** Confirm the disk model, capacity, serial or other identity details. Read the current MBR/GPT table before choosing a layout. Use an image file for practice and automated testing.
3. **Choose a scenario.** Select **Windows**, **Linux**, **macOS**, or **Multiboot**, then choose the matching layout and capacity options. The scenario names are the user-facing choice; template identifiers are implementation details.
4. **Preview the partition draft.** Review the partition table type, start and end sectors, sizes, alignment, filesystem/type notes, and the diff from the current disk. No write should happen during preview.
5. **Back up before writing.** Create the `.rspbak` backup and keep a copy somewhere other than the target disk. Check the backup metadata and disk identity.
6. **Dry-run and explicitly confirm.** Recheck the target and the final diff. Only the explicit confirmation step may authorize a permitted write. Cancel if the disk identity, layout, or intended OS is unclear.
7. **Hand off to the OS installer.** Once the target has been prepared, reboot or power off, boot the official Windows, Linux, or macOS installer, select the intended prepared space, and follow that installer. For multiboot, install each OS separately and verify its target before continuing.

### Scenario Handoff

| Scenario | What SPFDisk prepares | What the OS installer completes |
|----------|-----------------------|---------------------------------|
| Windows | Usually GPT/UEFI with an ESP, MSR, Windows target, and optional recovery/data space | Windows files, filesystem setup as requested, Windows Boot Manager, and recovery configuration |
| Linux | An ESP or BIOS boot partition plus root, home, and swap space according to the selected layout | Linux files, filesystem setup as requested, and GRUB/systemd-boot installation and configuration |
| macOS | A GPT/GUID Partition Map layout with an Apple APFS target partition and optional shared space | macOS installation, APFS formatting, volumes, recovery, and Apple boot components |
| Multiboot | Shared or selected ESP space plus separate areas for each operating system | Each official installer installs its own OS and bootloader; shared ESP preparation alone installs no bootloader |

**macOS boundary:** SPFDisk does not format APFS. The macOS installer or Disk Utility must format and set up APFS. SPFDisk also does not configure Hackintosh/OpenCore, FileVault, or macOS recovery.

### Safety Requirements

Every layout change follows this order:

```text
Snapshot → Draft → Preview → Backup → Dry Run → Explicit Confirmation → Write / Rollback
```

- The default mode is read-only or preview-only. A backup and a preview do not make the wrong target safe.
- Development and automated tests use image files. Never use a real system disk as a test image.
- Real-disk writes remain high risk and require the applicable safety gate, correct disk identity, backup stored elsewhere, and explicit confirmation. Do not select a disk by `/dev/sdX` or a drive letter alone.
- Close applications and unmount partitions where applicable before a real-disk operation. Stop when the current layout or target identity is not unambiguous.
- Do not assume that a successful SPFDisk write installed an OS or bootloader. The final boot files come from the corresponding OS installer.
- APFS targets are prepared but not formatted by SPFDisk.

### CLI and Image Quick Start

The CLI is useful for repeatable image-based previews and tests:

```bash
# Build
cargo build --release -p rspfdisk-cli

# Preview a Windows partition layout (dry-run, no write)
./target/release/rspfdisk layout windows-standard test.img --dry-run

# Write GPT to an image only after an explicit image confirmation
./target/release/rspfdisk layout windows-standard test.img \
  --write --yes-i-know-this-is-an-image

# Inspect the resulting partition table
./target/release/rspfdisk inspect test.img

# Launch the TUI
./target/release/rspfdisk tui
./target/release/rspfdisk tui --image test.img
```

### Further Reading

- [Bootable Guided Install](docs/bootable-guided-install.md) — newcomer checklist and OS handoff
- [Boot media](docs/boot-media.md) — ISO, USB, and QEMU boot path
- [Quick layouts](docs/quick-layouts.md) — draft and alignment rules
- [Windows layout](docs/windows-layout.md), [Linux layout](docs/linux-layout.md), and [macOS layout](docs/macos-layout.md) — scenario details
- [Safety](docs/safety.md) — write and recovery constraints

### Test Suite

```bash
cargo test --workspace               # fast tests
cargo test --workspace -- --ignored  # slow image-write tests
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
