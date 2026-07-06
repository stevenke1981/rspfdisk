use thiserror::Error;

pub type BackupResult<T> = Result<T, BackupError>;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("disk error: {0}")]
    Disk(#[from] rspfdisk_disk::DiskError),
    #[error("gpt error: {0}")]
    Gpt(#[from] rspfdisk_gpt::GptError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("checksum mismatch")]
    ChecksumMismatch,
    #[error("disk identity mismatch: {0}")]
    IdentityMismatch(String),
    #[error("invalid backup format")]
    InvalidFormat,
}
