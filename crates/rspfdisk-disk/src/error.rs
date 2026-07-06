use rspfdisk_core::error::CoreError;
use thiserror::Error;

pub type DiskResult<T> = Result<T, DiskError>;

#[derive(Debug, Error)]
pub enum DiskError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "read beyond device boundary: lba={lba}, count={count}, device_sectors={device_sectors}"
    )]
    OutOfBounds {
        lba: u64,
        count: u64,
        device_sectors: u64,
    },
    #[error("device is read-only")]
    ReadOnly,
    #[error("invalid sector size")]
    InvalidSectorSize,
    #[error("device size not aligned to sector size")]
    MisalignedSize,
    #[error("platform unsupported: {0}")]
    PlatformUnsupported(String),
    #[error("insufficient privileges: {0}")]
    InsufficientPrivileges(String),
    #[error("invalid block device path: {0}")]
    InvalidBlockPath(String),
    #[error("core error: {0}")]
    Core(#[from] CoreError),
}
