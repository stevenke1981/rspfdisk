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
}

#[test]
fn grub_cfg_references_initramfs() {
    let cfg = std::fs::read_to_string(project_root().join("boot/grub/grub.cfg")).unwrap();
    assert!(cfg.contains("initramfs.img"));
    assert!(cfg.contains("vmlinuz"));
}

#[test]
fn qemu_ps1_supports_validate_only() {
    let script = std::fs::read_to_string(project_root().join("tools/qemu-test.ps1")).unwrap();
    assert!(script.contains("ValidateOnly"));
    assert!(script.contains("qemu-system-x86_64"));
}
