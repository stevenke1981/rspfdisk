use std::path::{Path, PathBuf};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("project root")
}

#[test]
fn boot_media_files_exist() {
    let root = project_root();
    let required = [
        "boot/initramfs/init",
        "boot/grub/grub.cfg",
        "boot/rspfdisk-media",
        "boot/quick-help.txt",
        "tools/make-boot-iso.sh",
        "tools/make-boot-iso.ps1",
        "tools/make-boot-usb.sh",
        "tools/make-boot-usb.ps1",
        "tools/qemu-test.sh",
        "tools/qemu-test.ps1",
        "tools/lib/boot-common.sh",
        // UEFI boot files
        "tools/build-uefi.sh",
        "tools/build-uefi.ps1",
        "tools/qemu-uefi-app.sh",
        "tools/qemu-uefi-smoke.sh",
        "dist/EFI/BOOT/BOOTX64.EFI",
        "dist/SHA256SUMS",
    ];
    for rel in required {
        let path = root.join(rel);
        assert!(path.exists(), "missing {rel}");
    }
}

#[test]
fn init_script_launches_tui() {
    let init = std::fs::read_to_string(project_root().join("boot/initramfs/init")).unwrap();
    assert!(init.contains("rspfdisk-tui"));
    assert!(init.contains("#!/bin/sh"));
    // Should mount sysfs and proc for device access
    assert!(
        init.contains("sysfs") || init.contains("/sys"),
        "init should mount sysfs"
    );
    assert!(
        init.contains("proc") || init.contains("/proc"),
        "init should mount proc"
    );
}

#[test]
fn grub_defaults_to_guided_tui_and_exposes_safe_recovery() {
    let cfg = std::fs::read_to_string(project_root().join("boot/grub/grub.cfg")).unwrap();
    let entries: Vec<&str> = cfg
        .lines()
        .filter(|line| line.trim_start().starts_with("menuentry "))
        .collect();

    assert_eq!(entries.len(), 2, "GRUB menu should stay beginner-friendly");
    assert!(cfg.contains("set timeout=5"));
    assert!(cfg.contains("set default=0"));
    assert!(entries[0].contains("guided TUI"));
    assert!(entries[0].contains("recommended"));
    assert!(entries[1].contains("Recovery shell"));
    assert!(entries[1].contains("advanced"));
    assert!(!entries[1].contains("read-only"));
    let linux_lines: Vec<&str> = cfg
        .lines()
        .filter(|line| line.trim_start().starts_with("linux "))
        .collect();
    assert_eq!(linux_lines.len(), 2);
    assert!(linux_lines.iter().all(|line| line.contains(" ro ")));
    assert!(cfg.contains("rspfdisk_mode=guided"));
    assert!(cfg.contains("rspfdisk_cli=1"));
    assert!(cfg.contains("search --no-floppy --file --set=root /rspfdisk-media"));
    assert!(linux_lines.iter().all(|line| line.contains("console=tty0")));
    assert!(
        linux_lines
            .iter()
            .all(|line| !line.contains("console=ttyS0")),
        "physical boot should keep the interactive TUI on the display"
    );
}

#[test]
fn init_routes_exact_kernel_cli_argument_to_recovery_shell() {
    let init = std::fs::read_to_string(project_root().join("boot/initramfs/init")).unwrap();
    assert!(init.contains("/proc/cmdline"));
    assert!(init.contains("kernel_cmdline"));
    assert!(init.contains("has_kernel_arg"));
    assert!(init.contains("rspfdisk_cli=1"));
    assert!(init.contains("case \" ${kernel_cmdline} \" in"));
    assert!(init.contains("RSPFDISK_RECOVERY_SHELL"));
    assert!(init.contains("exec /bin/sh"));
    assert!(init.contains("RSPFDISK_BOOT_MODE=recovery-cli"));
    assert!(
        !init
            .lines()
            .any(|line| line.trim_start().starts_with("eval ")),
        "kernel command line must never be evaluated as shell code"
    );
}

#[test]
fn boot_runtime_emits_markers_at_ready_points() {
    let root = project_root();
    let init = std::fs::read_to_string(root.join("boot/initramfs/init")).unwrap();
    let recovery_marker = init
        .find("emit_marker \"RSPFDISK_RECOVERY_SHELL\"")
        .unwrap();
    let recovery_exec = init.find("exec /bin/sh").unwrap();

    assert!(recovery_marker < recovery_exec);
    assert!(init.contains("RSPFDISK_RECOVERY_SHELL"));
    assert!(init.contains("> /dev/ttyS0"));

    let tui = std::fs::read_to_string(root.join("crates/rspfdisk-tui/src/lib.rs")).unwrap();
    let first_draw = tui.find("terminal.draw").unwrap();
    let ready_call = tui.find("emit_boot_ready_marker();").unwrap();
    assert!(first_draw < ready_call);
    assert!(tui.contains("RSPFDISK_TUI_READY"));
    assert!(tui.contains("/dev/ttyS0"));
}

#[test]
fn initramfs_packaging_includes_required_runtime() {
    let common = std::fs::read_to_string(project_root().join("tools/lib/boot-common.sh")).unwrap();
    for required in [
        "install_busybox",
        "copy_runtime_dependencies",
        "${stage}/dev",
        "${stage}/proc",
        "${stage}/sys",
        "${stage}/run",
    ] {
        assert!(
            common.contains(required),
            "missing runtime step: {required}"
        );
    }

    let iso = std::fs::read_to_string(project_root().join("tools/make-boot-iso.sh")).unwrap();
    assert!(iso.contains("require_cmd grub-mkrescue"));
    assert!(!iso.contains("no bootloader embedding"));
}

#[test]
fn qemu_smoke_requires_a_serial_ready_marker() {
    let script = std::fs::read_to_string(project_root().join("tools/qemu-test.sh")).unwrap();
    assert!(script.contains("RSPFDISK_TUI_READY"));
    assert!(script.contains("grep -Fq"));
    assert!(script.contains("before serial marker"));
    assert!(!script.contains("may be OK for smoke"));
    assert!(script.contains("--usb"));
    assert!(script.contains("usb-storage,drive=rspfdisk_usb"));
}

#[test]
fn quick_help_matches_grub_flow_and_safe_cli_examples() {
    let help = std::fs::read_to_string(project_root().join("boot/quick-help.txt")).unwrap();
    assert!(help.contains("guided TUI"));
    assert!(help.contains("Recovery shell"));
    assert!(help.contains("rspfdisk list"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("預設唯讀"));
    assert!(!help.contains("[1] Windows"));
}

#[test]
fn grub_cfg_references_initramfs() {
    let cfg = std::fs::read_to_string(project_root().join("boot/grub/grub.cfg")).unwrap();
    assert!(cfg.contains("initramfs.img"));
    assert!(cfg.contains("vmlinuz"));
    assert!(
        cfg.contains("rspfdisk"),
        "GRUB config should reference rspfdisk"
    );
}

#[test]
fn qemu_ps1_supports_validate_only() {
    let script = std::fs::read_to_string(project_root().join("tools/qemu-test.ps1")).unwrap();
    assert!(script.contains("ValidateOnly"));
    assert!(script.contains("qemu-system-x86_64"));
    assert!(script.contains("ConvertTo-WslPath"));
    assert!(script.contains("Assert-SerialMarker"));
}

#[test]
fn usb_builder_creates_a_rootless_uefi_grub_image() {
    let script = std::fs::read_to_string(project_root().join("tools/make-boot-usb.sh")).unwrap();
    for required in [
        "grub-mkstandalone",
        "BOOTX64.EFI",
        "boot/vmlinuz",
        "boot/initramfs.img",
        "ESP_IMAGE",
        "verify_esp_files",
        "@@${ESP_OFFSET_BYTES}",
        "${MEDIA_MARKER}",
    ] {
        assert!(script.contains(required), "missing USB step: {required}");
    }
    assert!(
        !script.contains("losetup"),
        "image generation should not require root or loop devices"
    );
}

#[test]
fn uefi_smoke_script_exists() {
    let script = project_root().join("tools/qemu-uefi-smoke.sh");
    assert!(script.exists(), "qemu-uefi-smoke.sh must exist for CI");
    let content = std::fs::read_to_string(&script).unwrap();
    assert!(
        content.contains("BOOTX64.EFI"),
        "script should reference BOOTX64.EFI"
    );
    assert!(
        content.contains("qemu-system-x86_64"),
        "script should launch QEMU"
    );
    assert!(
        content.contains("OVMF"),
        "script should reference OVMF firmware"
    );
}

#[test]
fn uefi_app_script_exists() {
    let script = project_root().join("tools/qemu-uefi-app.sh");
    assert!(script.exists(), "qemu-uefi-app.sh must exist");
    let content = std::fs::read_to_string(&script).unwrap();
    assert!(content.contains("BOOTX64.EFI"));
    assert!(content.contains("qemu-system-x86_64"));
}

#[test]
fn boot_iso_bundle_structure() {
    let root = project_root();
    // Verify all files needed to build a boot ISO are present
    let bundle = [
        "boot/initramfs/init",
        "boot/grub/grub.cfg",
        "boot/quick-help.txt",
        "tools/make-boot-iso.sh",
        "tools/lib/boot-common.sh",
        "crates/rspfdisk-tui/src/main.rs",
    ];
    for rel in bundle {
        let path = root.join(rel);
        assert!(path.exists(), "ISO bundle missing {rel}");
    }
}

#[test]
fn release_scripts_exist() {
    let root = project_root();
    for script in [
        "tools/make-release.sh",
        "tools/make-release.ps1",
        "tools/verify-linux.sh",
    ] {
        let path = root.join(script);
        assert!(path.exists(), "missing {script}");
    }
}

#[test]
fn linux_iso_acceptance_cannot_silently_skip_the_build() {
    let root = project_root();
    let verify = std::fs::read_to_string(root.join("tools/verify-linux.sh")).unwrap();
    assert!(verify.contains("bash tools/make-boot-iso.sh"));
    assert!(!verify.contains("SKIP: no kernel"));

    let workflow =
        std::fs::read_to_string(root.join(".github/workflows/linux-acceptance.yml")).unwrap();
    assert!(workflow.contains("linux-image-generic"));
    assert!(workflow.contains("bash tools/make-boot-usb.sh"));
    assert!(workflow.contains("bash tools/qemu-test.sh --bios"));
    assert!(workflow.contains("bash tools/qemu-test.sh --uefi --usb"));
}

#[test]
fn sha256sums_file_exists() {
    let sums = project_root().join("dist/SHA256SUMS");
    assert!(sums.exists(), "dist/SHA256SUMS must exist for release");
    let content = std::fs::read_to_string(&sums).unwrap();
    assert!(!content.is_empty(), "SHA256SUMS should not be empty");
    // Each line should contain a hash and filename
    for line in content.lines() {
        assert!(
            line.len() > 64,
            "each SHA256SUMS line should contain a hash: {line}"
        );
    }
}
