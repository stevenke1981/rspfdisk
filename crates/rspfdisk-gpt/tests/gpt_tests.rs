use rspfdisk_core::{DiskInfo, SectorSize};
use rspfdisk_disk::{create_test_image, BlockDevice, FileBlockDevice};
use rspfdisk_gpt::{parse_gpt, validate_alignment, write_gpt_from_draft};
use rspfdisk_layouts::{generate_layout, load_template};

fn workspace_image(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/images")
        .join(name)
}

#[test]
fn write_and_read_gpt() {
    let path = workspace_image("gpt-roundtrip.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut dev = create_test_image(&path, 8 * 1024 * 1024 * 1024).unwrap();

    let template = load_template("../../templates/windows_uefi_standard.toml").unwrap();
    let disk = DiskInfo {
        path: path.display().to_string(),
        size_bytes: dev.size_bytes(),
        logical_sector_size: SectorSize::S512,
        physical_sector_size: Some(SectorSize::S512),
        model: None,
        serial: None,
        removable: false,
        read_only: false,
    };
    let draft = generate_layout(&template, &disk, None).unwrap();
    write_gpt_from_draft(&mut dev, &draft).unwrap();

    let dev = FileBlockDevice::open_read_only(path, SectorSize::S512).unwrap();
    let table = parse_gpt(&dev).unwrap();
    assert_eq!(table.partitions.len(), 4);
    for part in &table.partitions {
        validate_alignment(part.start_lba, SectorSize::S512).unwrap();
    }
}

#[test]
fn empty_image_has_no_gpt() {
    let path = workspace_image("gpt-empty-small.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    create_test_image(&path, 512 * 100).unwrap();
    let dev = FileBlockDevice::open_read_only(&path, SectorSize::S512).unwrap();
    assert!(parse_gpt(&dev).is_err());
}
