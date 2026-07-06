use rspfdisk_disk::{
    classify_path, is_linux_disk_write_candidate, linux_block_name, DevicePathKind,
};

#[test]
fn classify_image_file() {
    assert_eq!(classify_path("test-empty.img"), DevicePathKind::ImageFile);
}

#[test]
fn classify_linux_block_device() {
    assert_eq!(
        classify_path("/dev/nvme0n1"),
        DevicePathKind::LinuxBlockDevice
    );
    assert_eq!(classify_path("/dev/sda"), DevicePathKind::LinuxBlockDevice);
}

#[test]
fn whole_disk_write_candidates() {
    assert!(is_linux_disk_write_candidate("/dev/sda"));
    assert!(is_linux_disk_write_candidate("/dev/nvme0n1"));
    assert!(!is_linux_disk_write_candidate("/dev/sda1"));
    assert!(!is_linux_disk_write_candidate("/dev/nvme0n1p2"));
}

#[test]
fn linux_block_name_parsing() {
    assert_eq!(linux_block_name("/dev/nvme0n1").as_deref(), Some("nvme0n1"));
    assert_eq!(linux_block_name("/dev/sr0").as_deref(), None);
}
