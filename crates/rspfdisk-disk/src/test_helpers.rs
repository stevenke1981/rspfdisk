use std::path::Path;

use rspfdisk_core::SectorSize;

use crate::file_device::FileBlockDevice;
use crate::DiskResult;

/// Create a sparse test image file of the given size.
pub fn create_test_image(path: impl AsRef<Path>, size_bytes: u64) -> DiskResult<FileBlockDevice> {
    FileBlockDevice::create_image(path, size_bytes, SectorSize::S512)
}
