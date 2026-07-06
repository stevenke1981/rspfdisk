use rspfdisk_core::{DiskInfo, SectorSize};

use crate::error::DiskResult;
use crate::sector_buf::SectorBuf;

pub trait BlockDevice {
    fn info(&self) -> DiskInfo;
    fn sector_size(&self) -> SectorSize;
    fn sector_count(&self) -> u64;
    fn size_bytes(&self) -> u64 {
        self.sector_count() * self.sector_size().bytes() as u64
    }

    fn read_sector(&self, lba: u64) -> DiskResult<SectorBuf> {
        self.read_sectors(lba, 1)
    }

    fn read_sectors(&self, lba: u64, count: u64) -> DiskResult<SectorBuf>;
}

pub trait WritableBlockDevice: BlockDevice {
    fn write_sector(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()>;
    fn write_sectors(&mut self, lba: u64, data: &SectorBuf) -> DiskResult<()>;
    fn flush(&mut self) -> DiskResult<()> {
        Ok(())
    }
}
