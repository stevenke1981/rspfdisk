use thiserror::Error;

pub type SafetyResult<T> = Result<T, SafetyError>;

#[derive(Debug, Error)]
pub enum SafetyError {
    #[error("write not confirmed")]
    NotConfirmed,
    #[error("confirmation phrase mismatch")]
    PhraseMismatch,
    #[error("backup required before write")]
    BackupRequired,
    #[error("disk identity mismatch")]
    DiskIdentityMismatch,
    #[error("write flag not set; use --write")]
    WriteFlagRequired,
    #[error("image confirmation required; use --yes-i-know-this-is-an-image")]
    ImageConfirmationRequired,
    #[error("system disk write blocked; use --accept-system-disk-risk after backup")]
    SystemDiskBlocked,
    #[error("write not allowed: {0}")]
    WriteNotAllowed(String),
    #[error("real disk write requires --confirm <disk-id>")]
    RealDiskConfirmRequired,
}
