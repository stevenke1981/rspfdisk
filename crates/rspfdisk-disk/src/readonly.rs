use rspfdisk_core::{DiskInfo, SectorSize};

use crate::device::BlockDevice;
use crate::error::{DiskError, DiskResult};
use crate::sector_buf::SectorBuf;

pub struct ReadOnlyDevice<D: BlockDevice> {
    inner: D,
}

impl<D: BlockDevice> ReadOnlyDevice<D> {
    pub fn new(inner: D) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> D {
        self.inner
    }
}

impl<D: BlockDevice> BlockDevice for ReadOnlyDevice<D> {
    fn info(&self) -> DiskInfo {
        let mut info = self.inner.info();
        info.read_only = true;
        info
    }

    fn sector_size(&self) -> SectorSize {
        self.inner.sector_size()
    }

    fn sector_count(&self) -> u64 {
        self.inner.sector_count()
    }

    fn read_sectors(&self, lba: u64, count: u64) -> DiskResult<SectorBuf> {
        self.inner.read_sectors(lba, count)
    }
}

impl<D: BlockDevice> ReadOnlyDevice<D> {
    pub fn write_denied(&self) -> DiskResult<()> {
        Err(DiskError::ReadOnly)
    }
}
