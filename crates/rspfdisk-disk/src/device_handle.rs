use rspfdisk_core::{DiskInfo, SectorSize};

use crate::device::{BlockDevice, WritableBlockDevice};
use crate::error::DiskResult;
use crate::file_device::FileBlockDevice;
use crate::sector_buf::SectorBuf;

#[cfg(target_os = "linux")]
use crate::linux_device::LinuxBlockDevice;

/// Unified block device handle (image file or Linux /dev node).
pub enum DeviceHandle {
    File(FileBlockDevice),
    #[cfg(target_os = "linux")]
    Linux(LinuxBlockDevice),
}

impl BlockDevice for DeviceHandle {
    fn info(&self) -> DiskInfo {
        match self {
            Self::File(d) => d.info(),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.info(),
        }
    }

    fn sector_size(&self) -> SectorSize {
        match self {
            Self::File(d) => d.sector_size(),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.sector_size(),
        }
    }

    fn sector_count(&self) -> u64 {
        match self {
            Self::File(d) => d.sector_count(),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.sector_count(),
        }
    }

    fn read_sectors(&self, lba: u64, count: u64) -> DiskResult<SectorBuf> {
        match self {
            Self::File(d) => d.read_sectors(lba, count),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.read_sectors(lba, count),
        }
    }
}

impl WritableBlockDevice for DeviceHandle {
    fn write_sector(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        match self {
            Self::File(d) => d.write_sector(lba, data),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.write_sector(lba, data),
        }
    }

    fn write_sectors(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        match self {
            Self::File(d) => d.write_sectors(lba, data),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.write_sectors(lba, data),
        }
    }

    fn flush(&mut self) -> DiskResult<()> {
        match self {
            Self::File(d) => d.flush(),
            #[cfg(target_os = "linux")]
            Self::Linux(d) => d.flush(),
        }
    }
}
