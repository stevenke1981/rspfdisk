use rspfdisk_core::{
    ChangePlan, DiffReport, DiskInfo, LayoutDraft, PartitionTableKind, SectorSize,
};
use rspfdisk_safety::{confirm_write, disk_confirmation_phrase, ConfirmationOptions};

fn sample_info(path: &str) -> DiskInfo {
    DiskInfo {
        path: path.to_string(),
        size_bytes: 8 * 1024 * 1024 * 1024,
        logical_sector_size: SectorSize::S512,
        physical_sector_size: Some(SectorSize::S512),
        model: None,
        serial: None,
        removable: true,
        read_only: false,
    }
}

fn sample_plan(path: &str) -> ChangePlan {
    ChangePlan {
        disk_path: path.to_string(),
        layout: LayoutDraft {
            template_name: "test".to_string(),
            display_name: "Test".to_string(),
            table: PartitionTableKind::Gpt,
            boot_mode: rspfdisk_core::BootMode::Uefi,
            partitions: vec![],
        },
        diff: DiffReport {
            creates_gpt: true,
            creates_mbr: false,
            added_partitions: vec![],
            removed_partitions: vec![],
            modified_partitions: vec![],
            summary_lines: vec![],
        },
        backup_path: None,
        dry_run: false,
    }
}

#[test]
fn no_write_flag_denied() {
    let plan = sample_plan("test.img");
    let opts = ConfirmationOptions {
        write: false,
        dry_run: true,
        image_confirmed: false,
        confirmation_phrase: None,
        backup_path: None,
        accept_system_disk_risk: false,
    };
    assert!(confirm_write(&plan, &opts, &sample_info("test.img")).is_err());
}

#[test]
fn image_write_requires_confirmation_flag() {
    let plan = sample_plan("test.img");
    let opts = ConfirmationOptions {
        write: true,
        dry_run: false,
        image_confirmed: false,
        confirmation_phrase: None,
        backup_path: None,
        accept_system_disk_risk: false,
    };
    assert!(confirm_write(&plan, &opts, &sample_info("test.img")).is_err());
}

#[test]
fn image_write_with_flags_succeeds() {
    let plan = sample_plan("test.img");
    let opts = ConfirmationOptions {
        write: true,
        dry_run: false,
        image_confirmed: true,
        confirmation_phrase: None,
        backup_path: Some("backup.rspbak".to_string()),
        accept_system_disk_risk: false,
    };
    assert!(confirm_write(&plan, &opts, &sample_info("test.img")).is_ok());
}

#[test]
fn real_disk_requires_confirm_phrase() {
    let plan = sample_plan("/dev/sdb");
    let info = sample_info("/dev/sdb");
    let opts = ConfirmationOptions {
        write: true,
        dry_run: false,
        image_confirmed: false,
        confirmation_phrase: None,
        backup_path: Some("backup.rspbak".to_string()),
        accept_system_disk_risk: false,
    };
    assert!(confirm_write(&plan, &opts, &info).is_err());
}

#[test]
fn real_disk_with_phrase_succeeds() {
    let plan = sample_plan("/dev/sdb");
    let info = sample_info("/dev/sdb");
    let opts = ConfirmationOptions {
        write: true,
        dry_run: false,
        image_confirmed: false,
        confirmation_phrase: Some("sdb".to_string()),
        backup_path: Some("backup.rspbak".to_string()),
        accept_system_disk_risk: false,
    };
    assert!(confirm_write(&plan, &opts, &info).is_ok());
}

#[test]
fn wrong_phrase_denied() {
    let plan = sample_plan("/dev/sda");
    let info = sample_info("/dev/sda");
    let opts = ConfirmationOptions {
        write: true,
        dry_run: false,
        image_confirmed: false,
        confirmation_phrase: Some("wrong".to_string()),
        backup_path: Some("backup.rspbak".to_string()),
        accept_system_disk_risk: true,
    };
    assert!(confirm_write(&plan, &opts, &info).is_err());
}

#[test]
fn disk_phrase_from_path() {
    assert_eq!(disk_confirmation_phrase("/dev/nvme0n1"), "nvme0n1");
}
