use thiserror::Error;

pub type GptResult<T> = Result<T, GptError>;

#[derive(Debug, Error)]
pub enum GptError {
    #[error("disk error: {0}")]
    Disk(#[from] rspfdisk_disk::DiskError),
    #[error("mbr error: {0}")]
    Mbr(#[from] rspfdisk_mbr::MbrError),
    #[error("invalid GPT signature")]
    InvalidSignature,
    #[error("invalid header CRC")]
    InvalidHeaderCrc,
    #[error("invalid partition entries CRC")]
    InvalidEntriesCrc,
    #[error("primary/backup GPT mismatch: {0}")]
    PrimaryBackupMismatch(String),
    #[error("partition overlap")]
    PartitionOverlap,
    #[error("alignment violation: {0}")]
    AlignmentViolation(String),
    #[error("invalid layout draft: {0}")]
    InvalidLayout(String),
    #[error("no GPT header found")]
    NoGptHeader,
}
