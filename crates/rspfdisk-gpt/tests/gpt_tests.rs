use rspfdisk_core::{
    BootMode, DiskInfo, LayoutDraft, PartitionDraft, PartitionTableKind, PartitionType, SectorSize,
};
use rspfdisk_disk::{create_test_image, BlockDevice, FileBlockDevice};
use rspfdisk_gpt::{parse_gpt, validate_alignment, write_gpt_from_draft, GptError};
use rspfdisk_layouts::{generate_layout, load_template};

fn workspace_image(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/images")
        .join(name)
}

#[test]
#[ignore = "slow 8GiB image write; covered by release gate with --include-ignored"]
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

#[test]
fn writer_preserves_draft_start_lba() {
    let path = workspace_image("gpt-custom-start.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut dev = create_test_image(&path, 256 * 1024 * 1024).unwrap();
    let draft = single_partition_draft(4096, PartitionTableKind::Gpt);

    write_gpt_from_draft(&mut dev, &draft).unwrap();

    let dev = FileBlockDevice::open_read_only(path, SectorSize::S512).unwrap();
    let table = parse_gpt(&dev).unwrap();
    assert_eq!(table.partitions.len(), 1);
    assert_eq!(table.partitions[0].start_lba, 4096);
}

#[test]
fn writer_rejects_non_gpt_draft() {
    let path = workspace_image("gpt-rejects-mbr-draft.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut dev = create_test_image(&path, 256 * 1024 * 1024).unwrap();
    let draft = single_partition_draft(2048, PartitionTableKind::Mbr);

    let err = write_gpt_from_draft(&mut dev, &draft).unwrap_err();
    assert!(matches!(err, GptError::InvalidLayout(_)));
}

#[test]
fn writer_rejects_overlapping_draft() {
    let path = workspace_image("gpt-rejects-overlap.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut dev = create_test_image(&path, 256 * 1024 * 1024).unwrap();
    let mut draft = single_partition_draft(2048, PartitionTableKind::Gpt);
    draft.partitions.push(PartitionDraft {
        name: "Overlap".to_string(),
        start_lba: 4096,
        size_bytes: 8 * 1024 * 1024,
        partition_type: PartitionType::LinuxFilesystem,
        filesystem: Some("ext4".to_string()),
        mount_point: None,
        note: None,
        flags: Vec::new(),
    });

    let err = write_gpt_from_draft(&mut dev, &draft).unwrap_err();
    assert!(matches!(err, GptError::InvalidLayout(_)));
}

fn single_partition_draft(start_lba: u64, table: PartitionTableKind) -> LayoutDraft {
    LayoutDraft {
        template_name: "test".to_string(),
        display_name: "Test".to_string(),
        table,
        boot_mode: BootMode::Uefi,
        partitions: vec![PartitionDraft {
            name: "Root".to_string(),
            start_lba,
            size_bytes: 64 * 1024 * 1024,
            partition_type: PartitionType::LinuxFilesystem,
            filesystem: Some("ext4".to_string()),
            mount_point: Some("/".to_string()),
            note: None,
            flags: Vec::new(),
        }],
    }
}
