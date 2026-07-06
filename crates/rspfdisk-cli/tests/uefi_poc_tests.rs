use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("project root")
}

#[test]
fn uefi_poc_files_exist() {
    let root = project_root();
    for rel in [
        "crates/rspfdisk-uefi/src/lib.rs",
        "crates/rspfdisk-uefi/src/main.rs",
        "crates/rspfdisk-uefi/src/uefi_entry.rs",
        "tools/build-uefi.sh",
        "tools/build-uefi.ps1",
        "tools/qemu-uefi-app.sh",
    ] {
        assert!(root.join(rel).exists(), "missing {rel}");
    }
}
