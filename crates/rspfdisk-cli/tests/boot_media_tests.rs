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
