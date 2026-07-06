//! Backup and restore (.rspbak format).

pub mod error;
pub mod format;
pub mod restore;
pub mod writer;

pub use error::{BackupError, BackupResult};
pub use format::BackupManifest;
pub use restore::{restore_dry_run, RestoreDiff};
pub use writer::create_backup;
