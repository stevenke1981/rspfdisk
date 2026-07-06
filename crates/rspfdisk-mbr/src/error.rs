use thiserror::Error;

pub type MbrResult<T> = Result<T, MbrError>;

#[derive(Debug, Error)]
pub enum MbrError {
    #[error("disk error: {0}")]
    Disk(#[from] rspfdisk_disk::DiskError),
    #[error("invalid MBR signature")]
    InvalidSignature,
    #[error("partition overlap")]
    PartitionOverlap,
    #[error("invalid partition entry")]
    InvalidEntry,
}
