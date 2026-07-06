use std::path::Path;

use rspfdisk_core::SectorSize;

use crate::device_handle::DeviceHandle;
use crate::error::{DiskError, DiskResult};
use crate::file_device::FileBlockDevice;
use crate::path_kind::{classify_path, DevicePathKind};

#[cfg(target_os = "linux")]
use crate::linux_device::LinuxBlockDevice;

pub fn open_read_only(path: impl AsRef<Path>) -> DiskResult<DeviceHandle> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    match classify_path(&path_str) {
        DevicePathKind::ImageFile | DevicePathKind::Unknown => Ok(DeviceHandle::File(
            FileBlockDevice::open_read_only(path, SectorSize::S512)?,
        )),
        DevicePathKind::LinuxBlockDevice => {
            #[cfg(target_os = "linux")]
            {
                Ok(DeviceHandle::Linux(LinuxBlockDevice::open_read_only(path)?))
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(DiskError::PlatformUnsupported(
                    "Linux block devices only supported on Linux".to_string(),
                ))
            }
        }
        DevicePathKind::WindowsPhysicalDrive => Err(DiskError::PlatformUnsupported(
            "Windows physical drive access is not implemented in v0.1".to_string(),
        )),
    }
}

pub fn open_read_write(path: impl AsRef<Path>) -> DiskResult<DeviceHandle> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    match classify_path(&path_str) {
        DevicePathKind::ImageFile | DevicePathKind::Unknown => Ok(DeviceHandle::File(
            FileBlockDevice::open_read_write(path, SectorSize::S512)?,
        )),
        DevicePathKind::LinuxBlockDevice => {
            #[cfg(target_os = "linux")]
            {
                crate::linux_permissions::require_root_for_write()?;
                Ok(DeviceHandle::Linux(LinuxBlockDevice::open_read_write(
                    path,
                )?))
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(DiskError::PlatformUnsupported(
                    "Linux block devices only supported on Linux".to_string(),
                ))
            }
        }
        DevicePathKind::WindowsPhysicalDrive => Err(DiskError::PlatformUnsupported(
            "Windows physical drive access is not implemented in v0.1".to_string(),
        )),
    }
}
