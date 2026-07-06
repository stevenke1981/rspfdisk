use rspfdisk_core::SectorSize;
use rspfdisk_disk::{create_test_image, BlockDevice, FileBlockDevice, WritableBlockDevice};
use tempfile::NamedTempFile;

#[test]
fn read_single_sector() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();
    let mut dev = create_test_image(path, 512 * 10).unwrap();
    let mut buf = rspfdisk_disk::SectorBuf::new(SectorSize::S512, 1);
    buf.as_bytes_mut()[0] = 0xAB;
    dev.write_sector(0, &buf).unwrap();
    dev.flush().unwrap();

    let dev = FileBlockDevice::open_read_only(path, SectorSize::S512).unwrap();
    let read = dev.read_sector(0).unwrap();
    assert_eq!(read.as_bytes()[0], 0xAB);
}

#[test]
fn read_multiple_sectors() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();
    create_test_image(path, 512 * 100).unwrap();
    let dev = FileBlockDevice::open_read_only(path, SectorSize::S512).unwrap();
    let read = dev.read_sectors(5, 3).unwrap();
    assert_eq!(read.sector_count(), 3);
}

#[test]
fn read_beyond_boundary_fails() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();
    create_test_image(path, 512 * 10).unwrap();
    let dev = FileBlockDevice::open_read_only(path, SectorSize::S512).unwrap();
    assert!(dev.read_sector(10).is_err());
    assert!(dev.read_sectors(8, 5).is_err());
}

#[test]
fn sector_size_4096() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();
    FileBlockDevice::create_image(path, 4096 * 8, SectorSize::S4096).unwrap();
    let dev = FileBlockDevice::open_read_only(path, SectorSize::S4096).unwrap();
    assert_eq!(dev.sector_count(), 8);
}
