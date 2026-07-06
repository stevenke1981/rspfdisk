use rspfdisk_core::SectorSize;
use rspfdisk_disk::{create_test_image, BlockDevice, FileBlockDevice, WritableBlockDevice};
use rspfdisk_gpt::header::GptHeader;
use rspfdisk_gpt::GptError;
use rspfdisk_gpt::{parse_gpt, write_gpt_from_draft};
use rspfdisk_layouts::{generate_layout, load_template};

fn workspace_image(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/images")
        .join(name)
}

fn write_valid_gpt(path: &std::path::Path) {
    let mut dev = create_test_image(path, 8 * 1024 * 1024 * 1024).unwrap();
    let template = load_template("../../templates/windows_uefi_standard.toml").unwrap();
    let disk = rspfdisk_core::DiskInfo {
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
}

#[test]
fn invalid_gpt_signature() {
    let path = workspace_image("gpt-negative-signature.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    write_valid_gpt(&path);

    let dev = FileBlockDevice::open_read_write(&path, SectorSize::S512).unwrap();
    let mut sector = dev.read_sector(1).unwrap();
    sector.as_bytes_mut()[0] = b'X';
    let mut writable = FileBlockDevice::open_read_write(&path, SectorSize::S512).unwrap();
    writable.write_sector(1, &sector).unwrap();

    let dev = FileBlockDevice::open_read_only(&path, SectorSize::S512).unwrap();
    let err = parse_gpt(&dev).unwrap_err();
    assert!(matches!(err, GptError::InvalidSignature));
}

#[test]
fn invalid_header_crc() {
    let path = workspace_image("gpt-negative-crc.img");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    write_valid_gpt(&path);

    let mut writable = FileBlockDevice::open_read_write(&path, SectorSize::S512).unwrap();
    let mut sector = writable.read_sector(1).unwrap();
    sector.as_bytes_mut()[20] ^= 0xFF;
    writable.write_sector(1, &sector).unwrap();

    let dev = FileBlockDevice::open_read_only(&path, SectorSize::S512).unwrap();
    let err = parse_gpt(&dev).unwrap_err();
    assert!(matches!(err, GptError::InvalidHeaderCrc));
}

#[test]
fn header_parse_rejects_short_buffer() {
    let err = GptHeader::parse(&[0u8; 64]).unwrap_err();
    assert!(matches!(err, GptError::NoGptHeader));
}
