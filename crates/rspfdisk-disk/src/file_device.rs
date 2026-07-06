use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use rspfdisk_core::{DiskInfo, SectorSize};

use crate::device::{BlockDevice, WritableBlockDevice};
use crate::error::{DiskError, DiskResult};
use crate::sector_buf::SectorBuf;

pub struct FileBlockDevice {
    file: File,
    path: PathBuf,
    sector_size: SectorSize,
    sector_count: u64,
    writable: bool,
}

impl FileBlockDevice {
    pub fn open_read_only(path: impl AsRef<Path>, sector_size: SectorSize) -> DiskResult<Self> {
        Self::open(path, sector_size, false)
    }

    pub fn open_read_write(path: impl AsRef<Path>, sector_size: SectorSize) -> DiskResult<Self> {
        Self::open(path, sector_size, true)
    }

    fn open(path: impl AsRef<Path>, sector_size: SectorSize, writable: bool) -> DiskResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = if writable {
            OpenOptions::new().read(true).write(true).open(&path)?
        } else {
            File::open(&path)?
        };

        let metadata = file.metadata()?;
        let size = metadata.len();
        let sector_bytes = sector_size.bytes() as u64;
        if size % sector_bytes != 0 {
            return Err(DiskError::MisalignedSize);
        }

        Ok(Self {
            file,
            path,
            sector_size,
            sector_count: size / sector_bytes,
            writable,
        })
    }

    pub fn create_image(
        path: impl AsRef<Path>,
        size_bytes: u64,
        sector_size: SectorSize,
    ) -> DiskResult<Self> {
        let sector_bytes = sector_size.bytes() as u64;
        let aligned_size = size_bytes.div_ceil(sector_bytes) * sector_bytes;
        let path = path.as_ref();
        let file = File::create(path)?;
        file.set_len(aligned_size)?;
        file.sync_all()?;
        drop(file);
        Self::open_read_write(path, sector_size)
    }

    fn check_bounds(&self, lba: u64, count: u64) -> DiskResult<()> {
        if lba.saturating_add(count) > self.sector_count {
            return Err(DiskError::OutOfBounds {
                lba,
                count,
                device_sectors: self.sector_count,
            });
        }
        Ok(())
    }
}

impl BlockDevice for FileBlockDevice {
    fn info(&self) -> DiskInfo {
        DiskInfo {
            path: self.path.display().to_string(),
            size_bytes: self.size_bytes(),
            logical_sector_size: self.sector_size,
            physical_sector_size: Some(self.sector_size),
            model: None,
            serial: None,
            removable: false,
            read_only: !self.writable,
        }
    }

    fn sector_size(&self) -> SectorSize {
        self.sector_size
    }

    fn sector_count(&self) -> u64 {
        self.sector_count
    }

    fn read_sectors(&self, lba: u64, count: u64) -> DiskResult<SectorBuf> {
        self.check_bounds(lba, count)?;
        let sector_bytes = self.sector_size.bytes() as usize;
        let mut buf = SectorBuf::new(self.sector_size, count as usize);
        let mut file = &self.file;
        file.seek(SeekFrom::Start(lba * sector_bytes as u64))?;
        file.read_exact(buf.as_bytes_mut())?;
        Ok(buf)
    }
}

impl WritableBlockDevice for FileBlockDevice {
    fn write_sector(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        self.write_sectors(lba, data)
    }

    fn write_sectors(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()> {
        if !self.writable {
            return Err(DiskError::ReadOnly);
        }
        let count = data.sector_count() as u64;
        self.check_bounds(lba, count)?;
        if data.sector_size() != self.sector_size {
            return Err(DiskError::InvalidSectorSize);
        }
        let sector_bytes = self.sector_size.bytes() as usize;
        self.file.seek(SeekFrom::Start(lba * sector_bytes as u64))?;
        self.file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn flush(&mut self) -> DiskResult<()> {
        self.file.sync_all()?;
        Ok(())
    }
}
