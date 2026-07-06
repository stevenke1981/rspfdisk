use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid sector size: {0}")]
    InvalidSectorSize(u32),
    #[error("alignment violation: {0}")]
    AlignmentViolation(String),
    #[error("partition overlap detected")]
    PartitionOverlap,
    #[error("insufficient disk space: {0}")]
    InsufficientSpace(String),
    #[error("invalid size expression: {0}")]
    InvalidSizeExpression(String),
    #[error("disk too small: need {need} bytes, have {have} bytes")]
    DiskTooSmall { need: u64, have: u64 },
}
