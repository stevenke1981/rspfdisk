use std::fs;
use std::path::PathBuf;

use rspfdisk_core::DiskInfo;

use crate::error::DiskResult;
use crate::linux_sysfs::query_block;
use crate::path_kind::is_allowed_linux_block_name;

pub fn list_linux_block_devices() -> DiskResult<Vec<DiskInfo>> {
    let mut disks = Vec::new();
    let block_dir = PathBuf::from("/sys/class/block");

    let mut entries: Vec<_> = fs::read_dir(&block_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| is_whole_disk(name))
        .collect();
    entries.sort();

    for name in entries {
        if !is_allowed_linux_block_name(&name) {
            continue;
        }
        let dev_path = PathBuf::from("/dev").join(&name);
        match query_block(&dev_path) {
            Ok(sysfs) => {
                disks.push(DiskInfo {
                    path: dev_path.display().to_string(),
                    size_bytes: sysfs.size_bytes,
                    logical_sector_size: sysfs.logical_sector_size,
                    physical_sector_size: sysfs.physical_sector_size,
                    model: sysfs.model,
                    serial: sysfs.serial,
                    removable: sysfs.removable,
                    read_only: sysfs.read_only,
                });
            }
            Err(_) => continue,
        }
    }

    Ok(disks)
}

fn is_whole_disk(name: &str) -> bool {
    if name.starts_with("nvme") {
        return !name.contains('p');
    }
    if name.starts_with("mmcblk") {
        return !name.contains('p');
    }
    if name.starts_with("sd") || name.starts_with("vd") {
        return name.chars().skip(2).all(|c| c.is_ascii_alphabetic());
    }
    false
}
