use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use rspfdisk_core::{DiskInfo, SectorSize};

use crate::device::{BlockDevice, WritableBlockDevice};
use crate::error::{DiskError, DiskResult};
use crate::linux_sysfs::{query_block, BlockSysfsInfo};
use crate::sector_buf::SectorBuf;

pub struct LinuxBlockDevice {
    file: File,
    path: PathBuf,
    sysfs: BlockSysfsInfo,
    writable: bool,
}

impl LinuxBlockDevice {
    pub fn open_read_only(path: impl AsRef<Path>) -> DiskResult<Self> {
        Self::open(path, false)
    }

    pub fn open_read_write(path: impl AsRef<Path>) -> DiskResult<Self> {
        Self::open(path, true)
    }

    fn open(path: impl AsRef<Path>, writable: bool) -> DiskResult<Self> {
        let path = path.as_ref().to_path_buf();
        let sysfs = query_block(&path)?;

        let file = if writable {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|e| DiskError::InsufficientPrivileges(e.to_string()))?
        } else {
            File::open(&path)?
        };

        Ok(Self {
            file,
            path,
            sysfs,
            writable,
        })
    }

    fn sector_bytes(&self) -> u64 {
        self.sysfs.logical_sector_size.bytes() as u64
    }

    fn computed_sectors(&self) -> u64 {
        self.sysfs.size_bytes / self.sector_bytes()
    }

    fn check_bounds(&self, lba: u64, count: u64) -> DiskResult<()> {
        let sectors = self.computed_sectors();
        if lba.saturating_add(count) > sectors {
            return Err(DiskError::OutOfBounds {
                lba,
                count,
                device_sectors: sectors,
            });
        }
        Ok(())
    }
}

impl BlockDevice for LinuxBlockDevice {
    fn info(&self) -> DiskInfo {
        DiskInfo {
            path: self.path.display().to_string(),
            size_bytes: self.sysfs.size_bytes,
            logical_sector_size: self.sysfs.logical_sector_size,
            physical_sector_size: self.sysfs.physical_sector_size,
            model: self.sysfs.model.clone(),
            serial: self.sysfs.serial.clone(),
            removable: self.sysfs.removable,
            read_only: self.sysfs.read_only || !self.writable,
        }
    }

    fn sector_size(&self) -> SectorSize {
        self.sysfs.logical_sector_size
    }

    fn sector_count(&self) -> u64 {
        self.computed_sectors()
    }

    fn read_sectors(&self, lba: u64, count: u64) -> DiskResult<SectorBuf> {
        self.check_bounds(lba, count)?;
        let sector_bytes = self.sector_bytes() as usize;
        let mut buf = SectorBuf::new(self.sysfs.logical_sector_size, count as usize);
        let mut file = &self.file;
        file.seek(SeekFrom::Start(lba * sector_bytes as u64))?;
        file.read_exact(buf.as_bytes_mut())?;
        Ok(buf)
    }
}

impl WritableBlockDevice for LinuxBlockDevice {
    fn write_sector(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        self.write_sectors(lba, data)
    }

    fn write_sectors(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        if !self.writable {
            return Err(DiskError::ReadOnly);
        }
        let count = data.sector_count() as u64;
        self.check_bounds(lba, count)?;
        if data.sector_size() != self.sysfs.logical_sector_size {
            return Err(DiskError::InvalidSectorSize);
        }
        let sector_bytes = self.sector_bytes() as usize;
        self.file.seek(SeekFrom::Start(lba * sector_bytes as u64))?;
        self.file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn flush(&mut self) -> DiskResult<()> {
        self.file.sync_all()?;
        Ok(())
    }
}
