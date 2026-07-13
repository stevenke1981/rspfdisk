# Progress - Bootable Guided Installer

## 2026-07-14

- Confirmed project root and required repository documents.
- Confirmed clean `master` at `cd32ad4` and indexed the repository with CBM.
- Read `planning-with-files` skill and ran session catchup.
- Inspected architecture, TUI symbols, boot-related scope, and prior release memory.
- Created persistent task plan, findings, and progress files.
- Read GRUB config, initramfs init, quick help, all template TOML files, README, and boot-media tests.
- Defined disjoint subagent ownership for boot assets/tests, TUI scenario flow, and onboarding documentation.
- Spawned three `gpt-5.6-luna xhigh` subagents for those disjoint areas.
- Confirmed `windows_uefi_dual_boot` is an add-Windows layout that assumes an existing ESP; planned a separate fresh Windows + Linux template.
- Added `templates/multiboot_windows_linux.toml` and focused layout tests.
- Found a P0 initramfs runtime packaging gap: missing shell, runtime directories, and dynamic ELF dependencies.
- Confirmed documentation currently overstates boot completion relative to the available QEMU evidence.
- Confirmed QEMU timeout-only smoke and USB packaging are additional evidence/bootability gaps.
- Assigned a fifth Luna xhigh subagent to harden QEMU marker verification and make the raw USB image genuinely UEFI/GRUB2 bootable or fail explicitly.
- Reviewed Singer's completed GRUB/init/help/tests slice and independently reran 13 boot-media tests successfully.
- Reviewed Peirce's initramfs runtime packaging; BusyBox, runtime directories, and ELF dependency copying are now explicit release prerequisites.
- Removed the non-bootable xorriso-only ISO fallback and kept the interactive TUI on the physical display while emitting QEMU readiness markers directly to serial.
- Hardened guided scenario selection so a target must be selected before a partition draft can be generated; removed automatic creation of a hidden fallback image.

## Verification Log

- `cargo test -p rspfdisk-layouts --test layout_tests fresh_windows_linux_multiboot_template` - passed (1 test).
- `cargo test -p rspfdisk-cli --test boot_media_tests` - passed (13 tests).
- `cargo test -p rspfdisk-tui` - passed (41 tests after target-selection, disk-cursor, and inspection-failure hardening).
- Relative Markdown links in both READMEs and the guided-install guide - passed.
- Direct `bash -n` was unavailable because this host's `bash` resolves to WSL without a distribution; retrying with Git Bash if installed.
- Git Bash syntax check passed for init and all modified boot shell scripts.
- PowerShell ISO, USB, and QEMU `-ValidateOnly` checks passed.
- Relative Markdown links passed after the spec/test/boot-media documentation update.
- Boot-media Rust tests passed (15 tests) after console, runtime, ISO, and marker hardening.
- Final boot-media suite passed with 17 tests, including rootless UEFI USB image and non-skippable Linux acceptance requirements.
- `cargo test --workspace` passed; 3 slow image tests were ignored in the default run.
- `cargo test --workspace -- --include-ignored` passed, including all 3 slow image-write cases.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo build --workspace --release` passed; Windows CLI and TUI release binaries were produced.
- Linux acceptance now installs a kernel and complete GRUB/USB/QEMU prerequisites, builds ISO and USB images, and boots the guided TUI in BIOS QEMU instead of silently skipping missing-kernel validation.
- Final Luna xhigh review found four issues; all were fixed and the same reviewer confirmed the resolutions, including UEFI USB QEMU coverage.
