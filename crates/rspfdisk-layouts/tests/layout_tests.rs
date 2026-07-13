use rspfdisk_core::{DiskInfo, SectorSize};
use rspfdisk_layouts::{generate_layout, load_template, TemplateRegistry};

fn disk_512g() -> DiskInfo {
    DiskInfo {
        path: "test.img".to_string(),
        size_bytes: 512 * 1024 * 1024 * 1024,
        logical_sector_size: SectorSize::S512,
        physical_sector_size: Some(SectorSize::S512),
        model: None,
        serial: None,
        removable: false,
        read_only: true,
    }
}

#[test]
fn windows_standard_template() {
    let template = load_template("../../templates/windows_uefi_standard.toml").unwrap();
    let draft = generate_layout(&template, &disk_512g(), None).unwrap();
    assert_eq!(draft.partitions.len(), 4);
    assert_eq!(draft.partitions[0].name, "EFI System");
    assert_eq!(draft.partitions[0].size_bytes, 512 * 1024 * 1024);
    assert_eq!(draft.partitions[1].name, "Microsoft Reserved");
    assert_eq!(draft.partitions[3].name, "Windows Recovery");
}

#[test]
fn linux_home_template() {
    let template = load_template("../../templates/linux_ext4_home.toml").unwrap();
    let draft = generate_layout(&template, &disk_512g(), None).unwrap();
    assert_eq!(draft.partitions.len(), 4);
    assert!(draft.partitions.iter().any(|p| p.name == "Linux Home"));
    assert!(draft.partitions.iter().any(|p| p.name == "Linux Swap"));
}

#[test]
fn macos_apfs_no_format() {
    let template = load_template("../../templates/macos_apfs_target.toml").unwrap();
    let draft = generate_layout(&template, &disk_512g(), None).unwrap();
    let macos = draft.partitions.iter().find(|p| p.name == "macOS").unwrap();
    assert_eq!(macos.filesystem.as_deref(), Some("none"));
}

#[test]
fn fresh_windows_linux_multiboot_template() {
    let template = load_template("../../templates/multiboot_windows_linux.toml").unwrap();
    let draft = generate_layout(&template, &disk_512g(), None).unwrap();

    assert_eq!(draft.partitions.len(), 6);
    assert_eq!(draft.partitions[0].name, "EFI System");
    assert!(draft.partitions[0].flags.iter().any(|flag| flag == "esp"));
    assert_eq!(draft.partitions[1].name, "Microsoft Reserved");
    assert_eq!(draft.partitions[2].name, "Windows");
    assert_eq!(draft.partitions[2].filesystem.as_deref(), Some("none"));
    assert_eq!(draft.partitions[3].name, "Linux Root");
    assert_eq!(draft.partitions[3].filesystem.as_deref(), Some("ext4"));
    assert_eq!(draft.partitions[5].name, "Shared Data");
    assert!(draft.partitions[5].size_bytes > 0);

    for pair in draft.partitions.windows(2) {
        let previous_end = pair[0].start_lba + pair[0].size_bytes / 512;
        assert!(
            previous_end <= pair[1].start_lba,
            "partitions must not overlap"
        );
    }
}

#[test]
fn small_disk_rejection() {
    let template = load_template("../../templates/windows_uefi_standard.toml").unwrap();
    let small = DiskInfo {
        size_bytes: 4 * 1024 * 1024 * 1024,
        ..disk_512g()
    };
    assert!(generate_layout(&template, &small, None).is_err());
}

#[test]
fn template_registry_loads_all() {
    let mut reg = TemplateRegistry::new();
    reg.load_dir("../../templates").unwrap();
    assert!(reg.get("windows_uefi_standard").is_ok());
    assert!(reg.get("linux_ext4_home").is_ok());
    assert!(reg.get("macos_apfs_target").is_ok());
    assert!(reg.get("multiboot_windows_linux").is_ok());
}
