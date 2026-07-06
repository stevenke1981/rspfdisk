use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectorSize(pub u32);

impl SectorSize {
    pub const S512: Self = Self(512);
    pub const S4096: Self = Self(4096);

    pub fn new(bytes: u32) -> CoreResult<Self> {
        match bytes {
            512 | 4096 => Ok(Self(bytes)),
            _ => Err(CoreError::InvalidSectorSize(bytes)),
        }
    }

    pub fn bytes(&self) -> u32 {
        self.0
    }

    pub fn align_lba(&self, lba: u64, align_bytes: u64) -> u64 {
        let sector_bytes = self.0 as u64;
        let byte_offset = lba * sector_bytes;
        let aligned = byte_offset.div_ceil(align_bytes) * align_bytes;
        aligned / sector_bytes
    }
}

/// Default 1 MiB alignment.
pub const ALIGN_1MIB: u64 = 1024 * 1024;
