use rspfdisk_core::SectorSize;
use rspfdisk_disk::{create_test_image, BlockDevice, FileBlockDevice};
use rspfdisk_mbr::{parse_mbr, parse_mbr_sector, write_protective_mbr};
use tempfile::NamedTempFile;

#[test]
fn empty_mbr() {
    let file = NamedTempFile::new().unwrap();
    create_test_image(file.path(), 512 * 100).unwrap();
    let dev = FileBlockDevice::open_read_only(file.path(), SectorSize::S512).unwrap();
    let table = parse_mbr(&dev).unwrap();
    assert!(table.partitions.is_empty());
}

#[test]
fn protective_mbr_write_and_read() {
    let file = NamedTempFile::new().unwrap();
    let mut dev = create_test_image(file.path(), 512 * 1000).unwrap();
    write_protective_mbr(&mut dev).unwrap();

    let dev = FileBlockDevice::open_read_only(file.path(), SectorSize::S512).unwrap();
    let sector = dev.read_sector(0).unwrap();
    let mbr = parse_mbr_sector(sector.as_bytes()).unwrap();
    assert!(mbr.is_protective);
    assert!(mbr.valid_signature);
}
