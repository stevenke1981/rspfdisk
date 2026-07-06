use std::fs;
use std::path::{Path, PathBuf};

use rspfdisk_core::SectorSize;

use crate::error::{DiskError, DiskResult};
use crate::path_kind::linux_block_name;

pub struct BlockSysfsInfo {
    pub block_name: String,
    pub size_bytes: u64,
    pub logical_sector_size: SectorSize,
    pub physical_sector_size: Option<SectorSize>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub removable: bool,
    pub read_only: bool,
}

pub fn sysfs_block_dir(name: &str) -> PathBuf {
    PathBuf::from("/sys/class/block").join(name)
}

pub fn read_sysfs_u64(path: impl AsRef<Path>) -> DiskResult<u64> {
    let raw = fs::read_to_string(path.as_ref())?;
    raw.trim().parse::<u64>().map_err(|_| {
        DiskError::InvalidBlockPath(format!("bad sysfs value at {}", path.as_ref().display()))
    })
}

pub fn read_sysfs_string(path: impl AsRef<Path>) -> Option<String> {
    let raw = fs::read_to_string(path.as_ref()).ok()?;
    let trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub fn query_block(path: &Path) -> DiskResult<BlockSysfsInfo> {
    let dev_path = path.to_string_lossy();
    let block_name = linux_block_name(&dev_path)
        .ok_or_else(|| DiskError::InvalidBlockPath(dev_path.to_string()))?;

    let base = sysfs_block_dir(&block_name);
    if !base.exists() {
        return Err(DiskError::InvalidBlockPath(format!(
            "sysfs entry not found: {}",
            base.display()
        )));
    }

    let kernel_sectors = read_sysfs_u64(base.join("size"))?;
    let logical = read_sysfs_u64(base.join("queue/logical_block_size"))
        .unwrap_or(512)
        .min(u32::MAX as u64) as u32;
    let logical_sector_size = SectorSize::new(logical)?;

    let physical = read_sysfs_u64(base.join("queue/physical_block_size"))
        .ok()
        .and_then(|v| SectorSize::new(v.min(u32::MAX as u64) as u32).ok());

    let size_bytes = kernel_sectors * 512;

    let removable = read_sysfs_u64(base.join("removable")).unwrap_or(0) == 1;
    let read_only = read_sysfs_u64(base.join("ro")).unwrap_or(0) == 1;

    let model = read_sysfs_string(base.join("device/model"))
        .or_else(|| read_sysfs_string(base.join("device/name")));
    let serial = read_sysfs_string(base.join("device/serial"))
        .or_else(|| read_sysfs_string(base.join("device/wwid")));

    Ok(BlockSysfsInfo {
        block_name,
        size_bytes,
        logical_sector_size,
        physical_sector_size: physical,
        model,
        serial,
        removable,
        read_only,
    })
}
