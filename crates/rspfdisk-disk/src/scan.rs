use rspfdisk_core::DiskInfo;

use crate::error::DiskResult;

#[cfg(target_os = "linux")]
pub fn list_block_devices() -> DiskResult<Vec<DiskInfo>> {
    crate::linux_scan::list_linux_block_devices()
}

#[cfg(not(target_os = "linux"))]
pub fn list_block_devices() -> DiskResult<Vec<DiskInfo>> {
    Ok(Vec::new())
}
