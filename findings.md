# Findings - Bootable Guided Installer

## Current State

- Repository is clean on `master` at `cd32ad4`, synchronized with `origin/master`.
- Existing boot assets include GRUB2 configuration, initramfs startup, ISO/USB builders, and QEMU validation scripts.
- TUI already has disk list, partition table, quick layout, preview, size editor, backup confirmation, and image-only write confirmation.
- Existing layout registry contains Windows, Linux, macOS, legacy, and dual-boot templates.
- Current user flow exposes implementation-oriented template names and keyboard commands instead of install scenarios.
- GRUB currently offers TUI and CLI shell entries, but the init script does not consume the `rspfdisk_cli=1` kernel argument.
- The quick help advertises direct Windows/macOS/Linux number keys that the current TUI main screen does not implement.
- Existing templates already cover single-OS, shared-data, legacy BIOS, and partial dual-boot layouts; the guided layer can map to them without changing the layout DSL.

## Product Decisions

- Treat SPFDisk as a partition preparation environment, not an OS installer.
- Use scenario language first, template names second.
- Keep GRUB2 menu short: start guided installer and an explicitly advanced recovery shell.
- Default automatically into the TUI so normal users do not need a shell.
- Multiboot scenarios must clearly distinguish shared ESP preparation from installing bootloaders.
- Use four top-level scenarios: Windows, Linux, macOS, and multiboot; each scenario expands to a short list of existing templates.
- Preserve an advanced template list for experienced users instead of removing current functionality.
- Fix the GRUB recovery path by making init parse `/proc/cmdline` and launch a shell when requested.
- Add a fresh-disk `multiboot_windows_linux` template instead of misusing `windows_uefi_dual_boot`, which assumes an ESP already exists.

## Planned Fresh Windows + Linux Layout

- GPT / UEFI.
- Shared 512 MiB EFI System Partition for Windows Boot Manager and GRUB2.
- 16 MiB Microsoft Reserved partition.
- 120 GiB Windows target, left unformatted for Windows Setup.
- 64 GiB Linux root target, marked ext4 for the Linux installer.
- Automatic swap partition.
- Remaining space as shared data; the installers still perform formatting and bootloader installation.

## Safety Notes

- TUI real-disk writes remain out of scope for automated work; tests use image files only.
- APFS partitions are prepared but not formatted.
- Windows/Linux/macOS installation media and GRUB installation are separate follow-up operations after partition preparation.

## P0 Boot Runtime Gap

- `assemble_initramfs_tree()` currently creates only `usr/bin`, `templates`, and `docs`.
- `/init` uses `#!/bin/sh`, `mount`, and `basename`, but the image does not include `/bin/sh` or BusyBox.
- The image does not pre-create `/dev`, `/proc`, or `/sys`, so early mounts can fail.
- GNU Rust release binaries are normally dynamically linked, but their ELF interpreter/shared libraries are not copied into the initramfs.
- A successful ISO file build therefore does not yet prove the initramfs can execute the TUI.
- `todos.md` marks Boot ISO design/scripts complete, while `final.md` still says real QEMU ISO boot is pending; completion language must distinguish bundle validation from executable boot proof.
- `qemu-test.sh` currently treats a timeout as success without checking a serial boot marker, so it cannot prove that `/init` or the TUI started.
- `make-boot-usb.sh` currently copies only initramfs/help into a FAT partition; it does not add a kernel or `EFI/BOOT/BOOTX64.EFI`, so the generated raw image is not a complete bootable USB.

## Boot Slice Review

- GRUB now defaults to a guided TUI entry and exposes a recovery shell entry.
- Init safely parses the exact `rspfdisk_cli=1` token without evaluating the command line.
- Stable `RSPFDISK_TUI_READY` and `RSPFDISK_RECOVERY_SHELL` markers are present.
- Integration adjustment needed: default output should favor `tty0` for physical users while writing only the readiness marker to `ttyS0` for headless QEMU evidence.

## Final Review Resolution

- GRUB root discovery uses the tracked `/rspfdisk-media` marker copied into both ISO and USB images.
- Recovery shell is labeled advanced; it no longer claims to enforce read-only behavior beyond normal CLI safety gates.
- Target inspection returns success/failure and blocks guided progression on unreadable or malformed partition tables.
- Linux CI boots both the BIOS ISO and UEFI USB image and requires the TUI-rendered serial readiness marker.
- Final Luna xhigh re-review confirmed all four findings resolved; the media marker is included in the staged commit.
