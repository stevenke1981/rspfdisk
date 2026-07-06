use rspfdisk_core::{DiskInfo, SectorSize};
use rspfdisk_safety::{assess_disk, validate_write_risk, RiskLevel};

fn sample_info(removable: bool) -> DiskInfo {
    DiskInfo {
        path: "/dev/sdb".to_string(),
        size_bytes: 64 * 1024 * 1024 * 1024,
        logical_sector_size: SectorSize::S512,
        physical_sector_size: Some(SectorSize::S512),
        model: Some("USB Disk".to_string()),
        serial: Some("ABC".to_string()),
        removable,
        read_only: false,
    }
}

#[test]
fn image_is_low_risk() {
    let info = DiskInfo {
        path: "test.img".to_string(),
        ..sample_info(true)
    };
    let risk = assess_disk("test.img", &info);
    assert!(risk.is_image);
    assert_eq!(risk.level, RiskLevel::Low);
    assert!(risk.write_allowed);
}

#[test]
fn fixed_disk_is_high_risk() {
    let info = sample_info(false);
    let risk = assess_disk("/dev/sda", &info);
    assert_eq!(risk.level, RiskLevel::High);
    assert!(!risk.is_image);
}

#[test]
fn partition_write_blocked() {
    let info = sample_info(true);
    let risk = assess_disk("/dev/sdb1", &info);
    assert_eq!(risk.level, RiskLevel::Critical);
    assert!(!risk.write_allowed);
    assert!(validate_write_risk("/dev/sdb1", &info, false).is_err());
}

#[test]
fn removable_whole_disk_write_allowed() {
    let info = sample_info(true);
    let risk = validate_write_risk("/dev/sdb", &info, false).unwrap();
    assert!(risk.write_allowed);
    assert!(risk.is_removable);
}
